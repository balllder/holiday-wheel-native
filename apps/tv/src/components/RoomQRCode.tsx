import React from 'react';
import { View, Text, StyleSheet } from 'react-native';
import QRCode from 'react-native-qrcode-svg';

interface RoomQRCodeProps {
  room: string;
  serverUrl: string;
  size?: number;
}

export function RoomQRCode({
  room,
  serverUrl,
  size = 200,
}: RoomQRCodeProps): React.JSX.Element {
  // Encode URL with room parameter for deep linking
  // Format: holidaywheel://join?room=ROOM_CODE&server=SERVER_URL
  const qrData = `holidaywheel://join?room=${encodeURIComponent(room)}&server=${encodeURIComponent(serverUrl)}`;

  return (
    <View style={styles.container}>
      <View style={styles.qrWrapper}>
        <QRCode
          value={qrData}
          size={size}
          backgroundColor="#1a0a3e"
          color="#d4af37"
        />
      </View>
      <Text style={styles.roomCode}>Room: {room}</Text>
      <Text style={styles.hint}>Scan with phone to join</Text>
      <Text style={styles.serverUrl}>{serverUrl}</Text>
    </View>
  );
}

const styles = StyleSheet.create({
  container: {
    alignItems: 'center',
    padding: 20,
    backgroundColor: '#1a0a3e',
    borderRadius: 12,
    borderWidth: 2,
    borderColor: '#d4af37',
  },
  qrWrapper: {
    padding: 16,
    backgroundColor: '#1a0a3e',
    borderRadius: 8,
  },
  roomCode: {
    color: '#d4af37',
    fontSize: 24,
    fontWeight: 'bold',
    marginTop: 16,
  },
  hint: {
    color: '#fff',
    fontSize: 18,
    marginTop: 8,
  },
  serverUrl: {
    color: '#888',
    fontSize: 14,
    marginTop: 8,
  },
});
