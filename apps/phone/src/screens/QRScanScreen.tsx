import React, { useCallback, useState } from 'react';
import {
  View,
  Text,
  StyleSheet,
  TouchableOpacity,
  Alert,
} from 'react-native';
import { Camera } from 'react-native-camera-kit';
import { NativeStackNavigationProp } from '@react-navigation/native-stack';
import { authService } from '@holiday-wheel/shared';
import type { RootStackParamList } from '../navigation/AppNavigator';

type QRScanScreenProps = {
  navigation: NativeStackNavigationProp<RootStackParamList, 'QRScan'>;
};

export function QRScanScreen({ navigation }: QRScanScreenProps): React.JSX.Element {
  const [scanned, setScanned] = useState(false);

  const onReadCode = useCallback(
    (event: { nativeEvent: { codeStringValue: string } }) => {
      if (scanned) return;

      try {
        const qrValue = event.nativeEvent.codeStringValue;

        // Parse the deep link URL
        // Format: holidaywheel://join?room=ROOM_CODE&server=SERVER_URL
        if (!qrValue.startsWith('holidaywheel://')) {
          Alert.alert('Invalid QR Code', 'This QR code is not a Holiday Wheel game code.');
          return;
        }

        setScanned(true);

        // Parse URL parameters manually (avoid URLSearchParams for RN compatibility)
        const urlParts = qrValue.replace('holidaywheel://join?', '');
        const paramsObj: Record<string, string> = {};
        urlParts.split('&').forEach((part) => {
          const [key, value] = part.split('=');
          if (key && value) {
            paramsObj[key] = decodeURIComponent(value);
          }
        });
        const room = paramsObj.room;
        const server = paramsObj.server;

        if (!room || !server) {
          Alert.alert('Invalid QR Code', 'Missing room or server information.');
          setScanned(false);
          return;
        }

        // Configure the server URL
        authService.setBaseUrl(server);

        // Navigate to controller screen with the room
        navigation.replace('Controller', { room });
      } catch {
        Alert.alert('Error', 'Failed to read QR code. Please try again.');
        setScanned(false);
      }
    },
    [navigation, scanned]
  );

  return (
    <View style={styles.container}>
      <Camera
        style={StyleSheet.absoluteFill}
        scanBarcode={true}
        onReadCode={onReadCode}
        showFrame={true}
        frameColor="#d4af37"
        laserColor="#d4af37"
      />

      <View style={styles.overlay}>
        <Text style={styles.title}>Scan QR Code</Text>
        <Text style={styles.subtitle}>
          Point your camera at the QR code on the TV screen
        </Text>
      </View>

      <TouchableOpacity
        style={styles.cancelButton}
        onPress={() => navigation.goBack()}
      >
        <Text style={styles.cancelText}>Cancel</Text>
      </TouchableOpacity>

      {scanned && (
        <View style={styles.scanningOverlay}>
          <Text style={styles.scanningText}>Joining game...</Text>
        </View>
      )}
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    flex: 1,
    backgroundColor: '#0d0628',
  },
  overlay: {
    position: 'absolute',
    top: 60,
    left: 0,
    right: 0,
    alignItems: 'center',
    padding: 20,
  },
  title: {
    color: '#d4af37',
    fontSize: 28,
    fontWeight: 'bold',
  },
  subtitle: {
    color: '#fff',
    fontSize: 16,
    marginTop: 8,
    textAlign: 'center',
  },
  cancelButton: {
    position: 'absolute',
    bottom: 40,
    left: 20,
    right: 20,
    backgroundColor: 'rgba(244, 67, 54, 0.9)',
    paddingVertical: 16,
    borderRadius: 12,
    alignItems: 'center',
  },
  cancelText: {
    color: '#fff',
    fontSize: 18,
    fontWeight: 'bold',
  },
  scanningOverlay: {
    position: 'absolute',
    top: 0,
    left: 0,
    right: 0,
    bottom: 0,
    backgroundColor: 'rgba(13, 6, 40, 0.9)',
    justifyContent: 'center',
    alignItems: 'center',
  },
  scanningText: {
    color: '#d4af37',
    fontSize: 24,
    fontWeight: 'bold',
  },
});
