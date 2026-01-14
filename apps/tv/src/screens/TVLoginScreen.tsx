import React, { useState, useRef } from 'react';
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
import { useAuthStore, authService } from '@holiday-wheel/shared';

const API_URL = 'http://192.168.1.100:5000'; // Update with your server IP

export function TVLoginScreen(): React.JSX.Element {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [focusedField, setFocusedField] = useState<'email' | 'password' | 'login'>('email');

  const emailRef = useRef<TextInput>(null);
  const passwordRef = useRef<TextInput>(null);

  const setAuth = useAuthStore((state) => state.setAuth);

  const handleLogin = async () => {
    if (!email.trim() || !password) {
      setError('Please enter email and password');
      return;
    }

    setLoading(true);
    setError(null);

    authService.setBaseUrl(API_URL);
    const result = await authService.login(email.trim().toLowerCase(), password);

    setLoading(false);

    if (result.ok && result.token && result.user) {
      await AsyncStorage.setItem('@auth_token', result.token);
      await AsyncStorage.setItem('@auth_user', JSON.stringify(result.user));
      setAuth(result.user, result.token);
    } else {
      setError(result.error || 'Login failed');
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

        {/* Login Form */}
        <TVFocusGuideView style={styles.form} autoFocus>
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
              hasTVPreferredFocus={true}
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

          {error && <Text style={styles.errorText}>{error}</Text>}

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
    marginBottom: 60,
  },
  logo: {
    fontSize: 120,
    marginBottom: 20,
  },
  title: {
    fontSize: 56,
    fontWeight: 'bold',
    color: '#d4af37',
  },
  subtitle: {
    fontSize: 36,
    color: '#ffd700',
    fontStyle: 'italic',
  },
  form: {
    width: 600,
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
    marginBottom: 16,
  },
  instructions: {
    marginTop: 60,
  },
  instructionText: {
    color: '#666',
    fontSize: 20,
  },
});
