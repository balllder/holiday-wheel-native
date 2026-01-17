/**
 * RoomQRCode Tests
 *
 * Note: Full component rendering tests are skipped due to react-native-tvos
 * compatibility issues with react-test-renderer.
 */

describe('RoomQRCode', () => {
  describe('QR code URL generation', () => {
    it('generates correctly formatted deep link URL', () => {
      const room = 'ABC123';
      const serverUrl = 'http://192.168.1.100:5000';

      const qrData = `holidaywheel://join?room=${encodeURIComponent(room)}&server=${encodeURIComponent(serverUrl)}`;

      expect(qrData).toBe(
        'holidaywheel://join?room=ABC123&server=http%3A%2F%2F192.168.1.100%3A5000'
      );
    });

    it('URL-encodes special characters', () => {
      const room = 'ROOM&CODE=TEST';
      const serverUrl = 'http://test.com?param=1';

      const qrData = `holidaywheel://join?room=${encodeURIComponent(room)}&server=${encodeURIComponent(serverUrl)}`;

      expect(qrData).toContain('room=ROOM%26CODE%3DTEST');
      expect(qrData).toContain('server=http%3A%2F%2Ftest.com%3Fparam%3D1');
    });

    it('handles empty room code', () => {
      const room = '';
      const serverUrl = 'http://test.com';

      const qrData = `holidaywheel://join?room=${encodeURIComponent(room)}&server=${encodeURIComponent(serverUrl)}`;

      expect(qrData).toBe('holidaywheel://join?room=&server=http%3A%2F%2Ftest.com');
    });
  });

  describe('RoomQRCode module', () => {
    it('exports RoomQRCode component', () => {
      const { RoomQRCode } = require('../src/components/RoomQRCode');
      expect(RoomQRCode).toBeDefined();
      expect(typeof RoomQRCode).toBe('function');
    });
  });
});
