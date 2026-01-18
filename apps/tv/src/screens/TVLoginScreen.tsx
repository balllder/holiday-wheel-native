import React, { useState, useRef, useCallback, useEffect } from 'react';
import {
  View,
  Text,
  TextInput,
  TouchableOpacity,
  StyleSheet,
  ActivityIndicator,
  TVFocusGuideView,
  Platform,
} from 'react-native';
import AsyncStorage from '@react-native-async-storage/async-storage';
import { useAuthStore, authService, oauthService } from '@holiday-wheel/shared';
import { configService } from '@holiday-wheel/shared';

export function TVLoginScreen(): React.JSX.Element {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [focusedField, setFocusedField] = useState<string>('google');
  const [showEmailForm, setShowEmailForm] = useState(false);

  const emailRef = useRef<TextInput>(null);
  const passwordRef = useRef<TextInput>(null);

  const setAuth = useAuthStore((state) => state.setAuth);

  // Initialize server URL from config
  useEffect(() => {
    const initUrl = async () => {
      const url = await configService.getServerUrl();
      authService.setBaseUrl(url);
    };
    initUrl();
  }, []);

  const handleAuthSuccess = useCallback(
    async (token: string, user: { id: number; email: string; display_name: string }) => {
      await AsyncStorage.setItem('@auth_token', token);
      await AsyncStorage.setItem('@auth_user', JSON.stringify(user));
      setAuth(user, token);
    },
    [setAuth]
  );

  const handleLogin = async () => {
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

  // Google Sign-In
  const handleGoogleSignIn = async () => {
    setLoading(true);
    setError(null);

    try {
      // TODO: Use @react-native-google-signin/google-signin for tvOS
      // const { idToken } = await GoogleSignin.signIn();
      // const result = await oauthService.googleAuth(idToken);

      // For now, show a message that Google Sign-In needs native setup
      setError(
        'Google Sign-In requires native SDK setup. ' +
          'Configure @react-native-google-signin/google-signin for tvOS.'
      );
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Google Sign-In error');
      setLoading(false);
    }
  };

  // Apple Sign-In (tvOS)
  const handleAppleSignIn = async () => {
    setLoading(true);
    setError(null);

    try {
      // TODO: Use @invertase/react-native-apple-authentication for tvOS
      // const appleAuthResponse = await appleAuth.performRequest({...});
      // const result = await oauthService.appleAuth(...);

      // For now, show a message that Apple Sign-In needs native setup
      setError(
        'Apple Sign-In requires native SDK setup. ' +
          'Enable Sign in with Apple capability for tvOS.'
      );
      setLoading(false);
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Apple Sign-In error');
      setLoading(false);
    }
  };

  return (
    <View style={styles.container}>
      <View style={styles.content}>
        {/* Logo */}
        <View style={styles.logoContainer}>
          <Text style={styles.logo}>🎡</Text>
          <Text style={styles.title}>Holiday Wheel</Text>
          <Text style={styles.subtitle}>of Fortune</Text>
        </View>

        {/* Login Options */}
        <TVFocusGuideView style={styles.form} autoFocus>
          {/* Social Login Buttons */}
          <TouchableOpacity
            style={[
              styles.socialButton,
              styles.googleButton,
              focusedField === 'google' && styles.buttonFocused,
            ]}
            onPress={handleGoogleSignIn}
            onFocus={() => setFocusedField('google')}
            disabled={loading}
            activeOpacity={0.8}
            hasTVPreferredFocus={!showEmailForm}
          >
            <Text style={styles.googleIcon}>G</Text>
            <Text style={styles.googleButtonText}>Sign in with Google</Text>
          </TouchableOpacity>

          {Platform.OS === 'ios' && (
            <TouchableOpacity
              style={[
                styles.socialButton,
                styles.appleButton,
                focusedField === 'apple' && styles.buttonFocused,
              ]}
              onPress={handleAppleSignIn}
              onFocus={() => setFocusedField('apple')}
              disabled={loading}
              activeOpacity={0.8}
            >
              <Text style={styles.appleIcon}></Text>
              <Text style={styles.appleButtonText}>Sign in with Apple</Text>
            </TouchableOpacity>
          )}

          {/* Divider */}
          <View style={styles.dividerContainer}>
            <View style={styles.divider} />
            <Text style={styles.dividerText}>or</Text>
            <View style={styles.divider} />
          </View>

          {/* Email/Password Form Toggle */}
          {!showEmailForm ? (
            <TouchableOpacity
              style={[
                styles.emailToggleButton,
                focusedField === 'emailToggle' && styles.buttonFocused,
              ]}
              onPress={() => setShowEmailForm(true)}
              onFocus={() => setFocusedField('emailToggle')}
              activeOpacity={0.8}
            >
              <Text style={styles.emailToggleText}>Sign in with email</Text>
            </TouchableOpacity>
          ) : (
            <>
              <View style={styles.inputContainer}>
                <Text style={styles.label}>Email</Text>
                <TextInput
                  ref={emailRef}
                  style={[
                    styles.input,
                    focusedField === 'email' && styles.inputFocused,
                  ]}
                  placeholder="Enter your email"
                  placeholderTextColor="#666"
                  value={email}
                  onChangeText={setEmail}
                  autoCapitalize="none"
                  keyboardType="email-address"
                  onFocus={() => setFocusedField('email')}
                  onSubmitEditing={() => passwordRef.current?.focus()}
                  hasTVPreferredFocus={showEmailForm}
                />
              </View>

              <View style={styles.inputContainer}>
                <Text style={styles.label}>Password</Text>
                <TextInput
                  ref={passwordRef}
                  style={[
                    styles.input,
                    focusedField === 'password' && styles.inputFocused,
                  ]}
                  placeholder="Enter your password"
                  placeholderTextColor="#666"
                  value={password}
                  onChangeText={setPassword}
                  secureTextEntry
                  onFocus={() => setFocusedField('password')}
                  onSubmitEditing={handleLogin}
                />
              </View>

              <TouchableOpacity
                style={[
                  styles.button,
                  focusedField === 'login' && styles.buttonFocused,
                  loading && styles.buttonDisabled,
                ]}
                onPress={handleLogin}
                onFocus={() => setFocusedField('login')}
                disabled={loading}
                activeOpacity={0.8}
              >
                {loading ? (
                  <ActivityIndicator color="#1a0a3e" size="large" />
                ) : (
                  <Text style={styles.buttonText}>LOGIN</Text>
                )}
              </TouchableOpacity>

              <TouchableOpacity
                style={[
                  styles.collapseButton,
                  focusedField === 'collapse' && styles.buttonFocused,
                ]}
                onPress={() => setShowEmailForm(false)}
                onFocus={() => setFocusedField('collapse')}
                activeOpacity={0.8}
              >
                <Text style={styles.collapseText}>Hide email form</Text>
              </TouchableOpacity>
            </>
          )}

          {error && <Text style={styles.errorText}>{error}</Text>}
        </TVFocusGuideView>

        {/* Instructions */}
        <View style={styles.instructions}>
          <Text style={styles.instructionText}>
            Use the remote to navigate • Press Select to interact
          </Text>
        </View>
      </View>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  content: {
    flex: 1,
    justifyContent: 'center',
    alignItems: 'center',
    padding: 80,
  },
  logoContainer: {
    alignItems: 'center',
    marginBottom: 50,
  },
  logo: {
    fontSize: 100,
    marginBottom: 16,
  },
  title: {
    fontSize: 48,
    fontWeight: 'bold',
    color: '#d4af37',
  },
  subtitle: {
    fontSize: 32,
    color: '#ffd700',
    fontStyle: 'italic',
  },
  form: {
    width: 600,
  },
  socialButton: {
    flexDirection: 'row',
    alignItems: 'center',
    justifyContent: 'center',
    borderRadius: 12,
    padding: 24,
    marginBottom: 16,
    borderWidth: 4,
    borderColor: 'transparent',
  },
  googleButton: {
    backgroundColor: '#ffffff',
  },
  appleButton: {
    backgroundColor: '#000000',
  },
  googleIcon: {
    fontSize: 28,
    fontWeight: 'bold',
    color: '#4285F4',
    marginRight: 16,
  },
  googleButtonText: {
    color: '#333',
    fontSize: 26,
    fontWeight: '600',
  },
  appleIcon: {
    color: '#ffffff',
    fontSize: 28,
    marginRight: 16,
  },
  appleButtonText: {
    color: '#ffffff',
    fontSize: 26,
    fontWeight: '600',
  },
  dividerContainer: {
    flexDirection: 'row',
    alignItems: 'center',
    marginVertical: 24,
  },
  divider: {
    flex: 1,
    height: 2,
    backgroundColor: '#333',
  },
  dividerText: {
    color: '#666',
    paddingHorizontal: 24,
    fontSize: 22,
  },
  emailToggleButton: {
    alignItems: 'center',
    padding: 24,
    borderRadius: 12,
    borderWidth: 3,
    borderColor: '#444',
  },
  emailToggleText: {
    color: '#888',
    fontSize: 24,
  },
  collapseButton: {
    alignItems: 'center',
    marginTop: 16,
    padding: 12,
    borderRadius: 8,
    borderWidth: 2,
    borderColor: 'transparent',
  },
  collapseText: {
    color: '#666',
    fontSize: 20,
  },
  inputContainer: {
    marginBottom: 24,
  },
  label: {
    fontSize: 24,
    color: '#fff',
    marginBottom: 8,
  },
  input: {
    backgroundColor: '#1a0a3e',
    borderRadius: 12,
    padding: 24,
    color: '#ffffff',
    fontSize: 28,
    borderWidth: 3,
    borderColor: '#333',
  },
  inputFocused: {
    borderColor: '#d4af37',
    backgroundColor: '#2a1a4e',
  },
  button: {
    backgroundColor: '#d4af37',
    borderRadius: 12,
    padding: 28,
    alignItems: 'center',
    marginTop: 24,
    borderWidth: 4,
    borderColor: 'transparent',
  },
  buttonFocused: {
    borderColor: '#fff',
    transform: [{ scale: 1.02 }],
  },
  buttonDisabled: {
    opacity: 0.7,
  },
  buttonText: {
    color: '#1a0a3e',
    fontSize: 32,
    fontWeight: 'bold',
  },
  errorText: {
    color: '#ff6b6b',
    fontSize: 22,
    textAlign: 'center',
    marginTop: 20,
    paddingHorizontal: 20,
  },
  instructions: {
    marginTop: 50,
  },
  instructionText: {
    color: '#666',
    fontSize: 20,
  },
});
