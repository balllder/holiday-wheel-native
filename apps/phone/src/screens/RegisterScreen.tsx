import React, { useState, useCallback } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  KeyboardAvoidingView,
  Platform,
  ScrollView,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import * as Passkeys from 'react-native-passkeys';
import {
  GoogleSignin,
  statusCodes,
  isErrorWithCode,
} from '@react-native-google-signin/google-signin';
import appleAuth from '@invertase/react-native-apple-authentication';
import { authService, passkeyService, oauthService, useAuthStore } from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type RegisterScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Register'>;
};

const API_URL = 'http://10.0.2.2:5000';

export function RegisterScreen({ navigation }: RegisterScreenProps): React.JSX.Element {
  const [email, setEmail] = useState('');
  const [displayName, setDisplayName] = useState('');
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [success, setSuccess] = useState(false);
  const [passkeySupported] = useState(Platform.OS === 'ios' || Platform.OS === 'android');

  const setAuth = useAuthStore((state) => state.setAuth);

  // Initialize services
  React.useEffect(() => {
    authService.setBaseUrl(API_URL);
    oauthService.setBaseUrl(API_URL);

    // Configure Google Sign-In
    GoogleSignin.configure({
      webClientId: 'YOUR_WEB_CLIENT_ID.apps.googleusercontent.com',
      offlineAccess: true,
      scopes: ['profile', 'email'],
    });
  }, []);

  const handleAuthSuccess = useCallback(
    async (token: string, user: { id: number; email: string; display_name: string }) => {
      await AsyncStorage.setItem('@auth_token', token);
      await AsyncStorage.setItem('@auth_user', JSON.stringify(user));
      setAuth(user, token);
    },
    [setAuth]
  );

  // Passkey registration
  const handlePasskeyRegister = async () => {
    if (!email.trim() || !displayName.trim()) {
      setError('Please enter email and display name first');
      return;
    }

    setLoading(true);
    setError(null);

    try {
      // Start passkey registration
      const startResult = await passkeyService.registerStart(
        email.trim().toLowerCase(),
        displayName.trim()
      );

      if (!startResult.ok || !startResult.options) {
        setError(startResult.error || 'Failed to start passkey registration');
        setLoading(false);
        return;
      }

      // Use react-native-passkeys to create the credential
      const credential = await Passkeys.create(startResult.options as any);

      if (!credential) {
        // User cancelled
        setLoading(false);
        return;
      }

      // Finish registration with the credential
      const finishResult = await passkeyService.registerFinish(
        email.trim().toLowerCase(),
        credential
      );

      if (finishResult.ok && finishResult.token && finishResult.user) {
        await handleAuthSuccess(finishResult.token, finishResult.user);
      } else {
        setError(finishResult.error || 'Passkey registration failed');
      }
    } catch (err: unknown) {
      const errorMessage = err instanceof Error ? err.message : 'Passkey error';
      // Check for common passkey error patterns
      if (errorMessage.toLowerCase().includes('cancel')) {
        // User cancelled, don't show error
      } else if (errorMessage.toLowerCase().includes('not supported')) {
        setError('Passkeys are not supported on this device');
      } else {
        setError(`Passkey error: ${errorMessage}`);
      }
    } finally {
      setLoading(false);
    }
  };

  const handleRegister = async () => {
    // Validation
    if (!email.trim() || !displayName.trim() || !password) {
      setError('Please fill in all fields');
      return;
    }

    if (password !== confirmPassword) {
      setError('Passwords do not match');
      return;
    }

    if (password.length < 6) {
      setError('Password must be at least 6 characters');
      return;
    }

    setLoading(true);
    setError(null);

    const result = await authService.register(
      email.trim().toLowerCase(),
      password,
      displayName.trim()
    );

    setLoading(false);

    if (result.ok) {
      setSuccess(true);
    } else {
      setError(result.error || 'Registration failed');
    }
  };

  // Google Sign-In (creates account automatically)
  const handleGoogleSignIn = async () => {
    setLoading(true);
    setError(null);

    try {
      await GoogleSignin.hasPlayServices();
      const signInResult = await GoogleSignin.signIn();

      const idToken = signInResult.data?.idToken;
      if (!idToken) {
        setError('Failed to get Google ID token');
        setLoading(false);
        return;
      }

      const result = await oauthService.googleAuth(idToken);

      if (result.ok && result.token && result.user) {
        await handleAuthSuccess(result.token, result.user);
      } else {
        setError(result.error || 'Google Sign-In failed');
      }
    } catch (err: unknown) {
      if (isErrorWithCode(err)) {
        switch (err.code) {
          case statusCodes.SIGN_IN_CANCELLED:
            break;
          case statusCodes.IN_PROGRESS:
            setError('Sign-in already in progress');
            break;
          case statusCodes.PLAY_SERVICES_NOT_AVAILABLE:
            setError('Google Play Services not available');
            break;
          default:
            setError(`Google Sign-In error: ${err.message}`);
        }
      } else {
        setError(err instanceof Error ? err.message : 'Google Sign-In error');
      }
    } finally {
      setLoading(false);
    }
  };

  // Apple Sign-In (creates account automatically, iOS only)
  const handleAppleSignIn = async () => {
    setLoading(true);
    setError(null);

    try {
      const appleAuthResponse = await appleAuth.performRequest({
        requestedOperation: appleAuth.Operation.LOGIN,
        requestedScopes: [appleAuth.Scope.EMAIL, appleAuth.Scope.FULL_NAME],
      });

      const credentialState = await appleAuth.getCredentialStateForUser(
        appleAuthResponse.user
      );

      if (credentialState !== appleAuth.State.AUTHORIZED) {
        setError('Apple Sign-In not authorized');
        setLoading(false);
        return;
      }

      const identityToken = appleAuthResponse.identityToken;
      if (!identityToken) {
        setError('Failed to get Apple identity token');
        setLoading(false);
        return;
      }

      // Convert null to undefined for fullName properties
      const fullName = appleAuthResponse.fullName
        ? {
            givenName: appleAuthResponse.fullName.givenName ?? undefined,
            familyName: appleAuthResponse.fullName.familyName ?? undefined,
          }
        : undefined;

      const result = await oauthService.appleAuth(
        identityToken,
        appleAuthResponse.user,
        appleAuthResponse.email ?? undefined,
        fullName
      );

      if (result.ok && result.token && result.user) {
        await handleAuthSuccess(result.token, result.user);
      } else {
        setError(result.error || 'Apple Sign-In failed');
      }
    } catch (err: unknown) {
      const appleError = err as { code?: string; message?: string };
      if (appleError.code === appleAuth.Error.CANCELED) {
        // User cancelled
      } else {
        setError(appleError.message || 'Apple Sign-In error');
      }
    } finally {
      setLoading(false);
    }
  };

  if (success) {
    return (
      <View style={styles.container}>
        <View style={styles.successContainer}>
          <Text style={styles.successIcon}>✉️</Text>
          <Text style={styles.successTitle}>Check Your Email</Text>
          <Text style={styles.successText}>
            We've sent a verification link to {email}. Please verify your email to
            continue.
          </Text>
          <TouchableOpacity
            style={styles.button}
            onPress={() => navigation.navigate('Login')}
          >
            <Text style={styles.buttonText}>Back to Login</Text>
          </TouchableOpacity>
        </View>
      </View>
    );
  }

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      style={styles.container}
    >
      <ScrollView contentContainerStyle={styles.scrollContent}>
        <View style={styles.formContainer}>
          <Text style={styles.header}>Create Account</Text>

          {/* Google Sign-In Button */}
          <TouchableOpacity
            style={[styles.googleButton, loading && styles.buttonDisabled]}
            onPress={handleGoogleSignIn}
            disabled={loading}
          >
            <Text style={styles.googleButtonIcon}>G</Text>
            <Text style={styles.googleButtonText}>Continue with Google</Text>
          </TouchableOpacity>

          {/* Apple Sign-In Button (iOS only) */}
          {Platform.OS === 'ios' && (
            <TouchableOpacity
              style={[styles.appleButton, loading && styles.buttonDisabled]}
              onPress={handleAppleSignIn}
              disabled={loading}
            >
              <Text style={styles.appleButtonIcon}>{'\uF8FF'}</Text>
              <Text style={styles.appleButtonText}>Continue with Apple</Text>
            </TouchableOpacity>
          )}

          {/* Passkey Registration Option */}
          {passkeySupported && (
            <TouchableOpacity
              style={[styles.passkeyButton, loading && styles.buttonDisabled]}
              onPress={handlePasskeyRegister}
              disabled={loading}
            >
              <Text style={styles.passkeyButtonIcon}>🔐</Text>
              <Text style={styles.passkeyButtonText}>Register with Passkey</Text>
            </TouchableOpacity>
          )}

          <View style={styles.dividerContainer}>
            <View style={styles.divider} />
            <Text style={styles.dividerText}>or register with email</Text>
            <View style={styles.divider} />
          </View>

          <TextInput
            style={styles.input}
            placeholder="Email"
            placeholderTextColor="#888"
            value={email}
            onChangeText={setEmail}
            autoCapitalize="none"
            keyboardType="email-address"
            autoComplete="email"
          />

          <TextInput
            style={styles.input}
            placeholder="Display Name"
            placeholderTextColor="#888"
            value={displayName}
            onChangeText={setDisplayName}
            autoCapitalize="words"
            autoComplete="name"
          />

          <TextInput
            style={styles.input}
            placeholder="Password"
            placeholderTextColor="#888"
            value={password}
            onChangeText={setPassword}
            secureTextEntry
            autoComplete="new-password"
          />

          <TextInput
            style={styles.input}
            placeholder="Confirm Password"
            placeholderTextColor="#888"
            value={confirmPassword}
            onChangeText={setConfirmPassword}
            secureTextEntry
            autoComplete="new-password"
          />

          {error && <Text style={styles.errorText}>{error}</Text>}

          <TouchableOpacity
            style={[styles.button, loading && styles.buttonDisabled]}
            onPress={handleRegister}
            disabled={loading}
          >
            {loading ? (
              <ActivityIndicator color="#1a0a3e" />
            ) : (
              <Text style={styles.buttonText}>Register</Text>
            )}
          </TouchableOpacity>

          <TouchableOpacity
            style={styles.linkButton}
            onPress={() => navigation.goBack()}
          >
            <Text style={styles.linkText}>
              Already have an account? <Text style={styles.linkTextBold}>Login</Text>
            </Text>
          </TouchableOpacity>
        </View>
      </ScrollView>
    </KeyboardAvoidingView>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  scrollContent: {
    flexGrow: 1,
    justifyContent: 'center',
    padding: 24,
  },
  formContainer: {
    width: '100%',
  },
  header: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#d4af37',
    textAlign: 'center',
    marginBottom: 32,
  },
  input: {
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
    padding: 16,
    marginBottom: 16,
    color: '#ffffff',
    fontSize: 16,
    borderWidth: 1,
    borderColor: '#333',
  },
  button: {
    backgroundColor: '#d4af37',
    borderRadius: 8,
    padding: 16,
    alignItems: 'center',
    marginTop: 8,
  },
  buttonDisabled: {
    opacity: 0.7,
  },
  buttonText: {
    color: '#1a0a3e',
    fontSize: 18,
    fontWeight: 'bold',
  },
  linkButton: {
    marginTop: 24,
    alignItems: 'center',
  },
  linkText: {
    color: '#888',
    fontSize: 14,
  },
  linkTextBold: {
    color: '#d4af37',
    fontWeight: 'bold',
  },
  errorText: {
    color: '#ff6b6b',
    textAlign: 'center',
    marginBottom: 16,
  },
  successContainer: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 24,
  },
  successIcon: {
    fontSize: 64,
    marginBottom: 24,
  },
  successTitle: {
    fontSize: 24,
    fontWeight: 'bold',
    color: '#d4af37',
    marginBottom: 16,
  },
  successText: {
    fontSize: 16,
    color: '#ccc',
    textAlign: 'center',
    marginBottom: 32,
    lineHeight: 24,
  },
  passkeyButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#4a3b8c',
    borderRadius: 8,
    padding: 14,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#6b5bb8',
  },
  passkeyButtonIcon: {
    fontSize: 20,
    marginRight: 10,
  },
  passkeyButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
  dividerContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 16,
  },
  divider: {
    flex: 1,
    height: 1,
    backgroundColor: '#333',
  },
  dividerText: {
    color: '#666',
    paddingHorizontal: 16,
    fontSize: 14,
  },
  googleButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#ffffff',
    borderRadius: 8,
    padding: 14,
    marginBottom: 12,
    borderWidth: 1,
    borderColor: '#ddd',
  },
  googleButtonIcon: {
    fontSize: 20,
    marginRight: 10,
    color: '#4285f4',
    fontWeight: 'bold',
  },
  googleButtonText: {
    color: '#333',
    fontSize: 16,
    fontWeight: '600',
  },
  appleButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    backgroundColor: '#000000',
    borderRadius: 8,
    padding: 14,
    marginBottom: 12,
  },
  appleButtonIcon: {
    fontSize: 20,
    marginRight: 10,
    color: '#ffffff',
  },
  appleButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
});
