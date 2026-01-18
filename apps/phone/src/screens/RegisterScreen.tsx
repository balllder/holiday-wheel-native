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
import { authService, passkeyService, useAuthStore } from '@holiday-wheel/shared';
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

          {/* Passkey Registration Option */}
          {passkeySupported && (
            <>
              <TouchableOpacity
                style={[styles.passkeyButton, loading && styles.buttonDisabled]}
                onPress={handlePasskeyRegister}
                disabled={loading}
              >
                <Text style={styles.passkeyButtonIcon}>🔐</Text>
                <Text style={styles.passkeyButtonText}>Register with Passkey</Text>
              </TouchableOpacity>

              <View style={styles.dividerContainer}>
                <View style={styles.divider} />
                <Text style={styles.dividerText}>or use password</Text>
                <View style={styles.divider} />
              </View>
            </>
          )}

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
});
