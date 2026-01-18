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
import {
  useAuthStore,
  authService,
  passkeyService,
} from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type LoginScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'Login'>;
};

const API_URL = 'http://10.0.2.2:5000'; // Android emulator localhost

export function LoginScreen({
  navigation,
}: LoginScreenProps): React.JSX.Element {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [showEmailForm, setShowEmailForm] = useState(false);
  const [passkeySupported] = useState(Platform.OS === 'ios' || Platform.OS === 'android');

  const setAuth = useAuthStore((state) => state.setAuth);

  // Initialize base URL
  React.useEffect(() => {
    authService.setBaseUrl(API_URL);
  }, []);

  const handleAuthSuccess = useCallback(
    async (token: string, user: { id: number; email: string; display_name: string }) => {
      await AsyncStorage.setItem('@auth_token', token);
      await AsyncStorage.setItem('@auth_user', JSON.stringify(user));
      setAuth(user, token);
    },
    [setAuth]
  );

  // Email/Password login
  const handleEmailLogin = async () => {
    if (!email.trim() || !password) {
      setError('Please enter email and password');
      return;
    }

    setLoading(true);
    setError(null);

    const result = await authService.login(email.trim().toLowerCase(), password);

    setLoading(false);

    if (result.ok && result.token && result.user) {
      await handleAuthSuccess(result.token, result.user);
    } else {
      setError(result.error || 'Login failed');
    }
  };

  // Passkey login
  const handlePasskeyLogin = async () => {
    setLoading(true);
    setError(null);

    try {
      // Start passkey authentication
      const startResult = await passkeyService.loginStart();
      if (!startResult.ok || !startResult.options) {
        setError(startResult.error || 'Failed to start passkey authentication');
        setLoading(false);
        return;
      }

      // TODO: Use react-native-passkeys to get the credential
      // const credential = await Passkey.get(startResult.options);

      // For now, show a message that passkeys need native setup
      setError(
        'Passkey authentication requires native SDK setup. ' +
          'Install react-native-passkeys and configure your app.'
      );
      setLoading(false);

      // When native SDK is set up, uncomment:
      // const finishResult = await passkeyService.loginFinish(credential);
      // if (finishResult.ok && finishResult.token && finishResult.user) {
      //   await handleAuthSuccess(finishResult.token, finishResult.user);
      // } else {
      //   setError(finishResult.error || 'Passkey authentication failed');
      // }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Passkey error');
      setLoading(false);
    }
  };

  // Google Sign-In
  const handleGoogleSignIn = async () => {
    setLoading(true);
    setError(null);

    try {
      // TODO: Use @react-native-google-signin/google-signin
      // const { idToken } = await GoogleSignin.signIn();
      // const result = await oauthService.googleAuth(idToken);

      // For now, show a message that Google Sign-In needs native setup
      setError(
        'Google Sign-In requires native SDK setup. ' +
          'Install @react-native-google-signin/google-signin and configure OAuth credentials.'
      );
      setLoading(false);

      // When native SDK is set up:
      // if (result.ok && result.token && result.user) {
      //   await handleAuthSuccess(result.token, result.user);
      // } else {
      //   setError(result.error || 'Google Sign-In failed');
      // }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Google Sign-In error');
      setLoading(false);
    }
  };

  // Apple Sign-In
  const handleAppleSignIn = async () => {
    setLoading(true);
    setError(null);

    try {
      // TODO: Use @invertase/react-native-apple-authentication
      // const appleAuthResponse = await appleAuth.performRequest({...});
      // const result = await oauthService.appleAuth(
      //   appleAuthResponse.identityToken,
      //   appleAuthResponse.user,
      //   appleAuthResponse.email,
      //   appleAuthResponse.fullName
      // );

      // For now, show a message that Apple Sign-In needs native setup
      setError(
        'Apple Sign-In requires native SDK setup. ' +
          'Install @invertase/react-native-apple-authentication and enable Sign in with Apple capability.'
      );
      setLoading(false);

      // When native SDK is set up:
      // if (result.ok && result.token && result.user) {
      //   await handleAuthSuccess(result.token, result.user);
      // } else {
      //   setError(result.error || 'Apple Sign-In failed');
      // }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Apple Sign-In error');
      setLoading(false);
    }
  };

  return (
    <KeyboardAvoidingView
      behavior={Platform.OS === 'ios' ? 'padding' : 'height'}
      style={styles.container}
    >
      <ScrollView contentContainerStyle={styles.scrollContent}>
        <View style={styles.logoContainer}>
          <Text style={styles.logo}>🎡</Text>
          <Text style={styles.title}>Holiday Wheel</Text>
          <Text style={styles.subtitle}>of Fortune</Text>
        </View>

        <View style={styles.formContainer}>
          {/* Passkey Button */}
          {passkeySupported && (
            <TouchableOpacity
              style={[styles.socialButton, styles.passkeyButton]}
              onPress={handlePasskeyLogin}
              disabled={loading}
            >
              <Text style={styles.socialButtonIcon}>🔐</Text>
              <Text style={styles.socialButtonText}>Sign in with Passkey</Text>
            </TouchableOpacity>
          )}

          {/* Google Sign-In Button */}
          <TouchableOpacity
            style={[styles.socialButton, styles.googleButton]}
            onPress={handleGoogleSignIn}
            disabled={loading}
          >
            <Text style={styles.socialButtonIcon}>G</Text>
            <Text style={styles.googleButtonText}>Sign in with Google</Text>
          </TouchableOpacity>

          {/* Apple Sign-In Button (iOS only) */}
          {Platform.OS === 'ios' && (
            <TouchableOpacity
              style={[styles.socialButton, styles.appleButton]}
              onPress={handleAppleSignIn}
              disabled={loading}
            >
              <Text style={styles.appleIcon} />
              <Text style={styles.appleButtonText}>Sign in with Apple</Text>
            </TouchableOpacity>
          )}

          {/* Divider */}
          <View style={styles.dividerContainer}>
            <View style={styles.divider} />
            <Text style={styles.dividerText}>or</Text>
            <View style={styles.divider} />
          </View>

          {/* Email/Password Toggle */}
          {!showEmailForm ? (
            <TouchableOpacity
              style={styles.emailToggleButton}
              onPress={() => setShowEmailForm(true)}
            >
              <Text style={styles.emailToggleText}>Sign in with email</Text>
            </TouchableOpacity>
          ) : (
            <>
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
                placeholder="Password"
                placeholderTextColor="#888"
                value={password}
                onChangeText={setPassword}
                secureTextEntry
                autoComplete="password"
              />

              <TouchableOpacity
                style={[styles.button, loading && styles.buttonDisabled]}
                onPress={handleEmailLogin}
                disabled={loading}
              >
                {loading ? (
                  <ActivityIndicator color="#1a0a3e" />
                ) : (
                  <Text style={styles.buttonText}>Login</Text>
                )}
              </TouchableOpacity>

              <TouchableOpacity
                style={styles.collapseButton}
                onPress={() => setShowEmailForm(false)}
              >
                <Text style={styles.collapseText}>Hide email form</Text>
              </TouchableOpacity>
            </>
          )}

          {error && <Text style={styles.errorText}>{error}</Text>}

          <TouchableOpacity
            style={styles.linkButton}
            onPress={() => navigation.navigate('Register')}
          >
            <Text style={styles.linkText}>
              Don't have an account?{' '}
              <Text style={styles.linkTextBold}>Register</Text>
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
  logoContainer: {
    alignItems: 'center',
    marginBottom: 36,
  },
  logo: {
    fontSize: 64,
    marginBottom: 16,
  },
  title: {
    fontSize: 32,
    fontWeight: 'bold',
    color: '#d4af37',
  },
  subtitle: {
    fontSize: 24,
    color: '#ffd700',
    fontStyle: 'italic',
  },
  formContainer: {
    width: '100%',
  },
  socialButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: 8,
    padding: 14,
    marginBottom: 12,
  },
  passkeyButton: {
    backgroundColor: '#4a3b8c',
    borderWidth: 1,
    borderColor: '#6b5bb8',
  },
  googleButton: {
    backgroundColor: '#ffffff',
    borderWidth: 1,
    borderColor: '#ddd',
  },
  appleButton: {
    backgroundColor: '#000000',
  },
  socialButtonIcon: {
    fontSize: 20,
    marginRight: 10,
  },
  socialButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
  googleButtonText: {
    color: '#333',
    fontSize: 16,
    fontWeight: '600',
  },
  appleIcon: {
    color: '#ffffff',
    fontSize: 20,
    marginRight: 10,
  },
  appleButtonText: {
    color: '#ffffff',
    fontSize: 16,
    fontWeight: '600',
  },
  dividerContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 20,
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
  emailToggleButton: {
    alignItems: 'center',
    padding: 14,
    borderRadius: 8,
    borderWidth: 1,
    borderColor: '#333',
  },
  emailToggleText: {
    color: '#888',
    fontSize: 16,
  },
  collapseButton: {
    alignItems: 'center',
    marginTop: 12,
  },
  collapseText: {
    color: '#666',
    fontSize: 14,
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
    marginTop: 16,
    marginBottom: 8,
    paddingHorizontal: 8,
  },
});
