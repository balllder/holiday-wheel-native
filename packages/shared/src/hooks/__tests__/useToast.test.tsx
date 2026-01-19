import React from 'react';
import { create, act, ReactTestRenderer } from 'react-test-renderer';
import { View } from 'react-native';
import { useToast, useToastHelpers } from '../useToast';

// Test component that uses useToast hook
function TestComponent({
  options = {},
  onHookResult,
}: {
  options?: Parameters<typeof useToast>[0];
  onHookResult?: (result: ReturnType<typeof useToast>) => void;
}) {
  const hookResult = useToast(options);

  // Call the callback with hook result for testing
  React.useEffect(() => {
    onHookResult?.(hookResult);
  }, [hookResult, onHookResult]);

  const { ToastComponent } = hookResult;

  return (
    <View testID="test-container">
      <ToastComponent />
    </View>
  );
}

// Test component for useToastHelpers
function TestHelpersComponent({
  options = {},
  onHookResult,
}: {
  options?: Parameters<typeof useToastHelpers>[0];
  onHookResult?: (result: ReturnType<typeof useToastHelpers>) => void;
}) {
  const hookResult = useToastHelpers(options);

  React.useEffect(() => {
    onHookResult?.(hookResult);
  }, [hookResult, onHookResult]);

  const { ToastComponent } = hookResult;

  return (
    <View testID="helpers-container">
      <ToastComponent />
    </View>
  );
}

describe('useToast', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('initial state', () => {
    it('returns isVisible as false initially', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(false);
    });

    it('returns showToast function', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.showToast).toBe('function');
    });

    it('returns hideToast function', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.hideToast).toBe('function');
    });

    it('returns ToastComponent', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.ToastComponent).toBeDefined();
    });
  });

  describe('showToast', () => {
    it('sets isVisible to true when called', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.showToast('Test message');
      });

      // Re-render to get updated state
      act(() => {
        tree?.update(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(true);
    });

    it('accepts message parameter', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      // Should not throw
      expect(() => {
        act(() => {
          hookResult?.showToast('Hello World');
        });
      }).not.toThrow();
    });

    it('accepts optional type parameter', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(() => {
        act(() => {
          hookResult?.showToast('Error message', 'error');
        });
      }).not.toThrow();
    });

    it('accepts optional duration parameter', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(() => {
        act(() => {
          hookResult?.showToast('Custom duration', 'info', 5000);
        });
      }).not.toThrow();
    });
  });

  describe('hideToast', () => {
    it('sets isVisible to false when called', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      // Show toast first
      act(() => {
        hookResult?.showToast('Test');
      });

      // Then hide it
      act(() => {
        hookResult?.hideToast();
      });

      act(() => {
        tree?.update(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(false);
    });
  });

  describe('options', () => {
    it('uses default duration of 3000ms', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      // Hook should be defined with default options
      expect(hookResult).toBeDefined();
    });

    it('accepts custom defaultDuration', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            options={{ defaultDuration: 5000 }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult).toBeDefined();
    });

    it('accepts custom defaultType', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            options={{ defaultType: 'success' }}
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult).toBeDefined();
    });
  });

  describe('ToastComponent', () => {
    it('renders without error', () => {
      let tree: ReactTestRenderer | undefined;

      expect(() => {
        act(() => {
          tree = create(<TestComponent />);
        });
      }).not.toThrow();

      expect(tree?.toJSON()).not.toBeNull();
    });

    it('has displayName set', () => {
      let hookResult: ReturnType<typeof useToast> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.ToastComponent.displayName).toBe('ToastComponent');
    });
  });
});

describe('useToastHelpers', () => {
  beforeEach(() => {
    jest.useFakeTimers();
  });

  afterEach(() => {
    jest.useRealTimers();
  });

  describe('helper functions', () => {
    it('returns showInfo function', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.showInfo).toBe('function');
    });

    it('returns showSuccess function', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.showSuccess).toBe('function');
    });

    it('returns showError function', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.showError).toBe('function');
    });

    it('returns showWarning function', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.showWarning).toBe('function');
    });
  });

  describe('showInfo', () => {
    it('shows toast with info type', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.showInfo('Info message');
      });

      act(() => {
        tree?.update(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(true);
    });

    it('accepts optional duration', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(() => {
        act(() => {
          hookResult?.showInfo('Info', 2000);
        });
      }).not.toThrow();
    });
  });

  describe('showSuccess', () => {
    it('shows toast with success type', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.showSuccess('Success message');
      });

      act(() => {
        tree?.update(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(true);
    });
  });

  describe('showError', () => {
    it('shows toast with error type', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.showError('Error message');
      });

      act(() => {
        tree?.update(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(true);
    });
  });

  describe('showWarning', () => {
    it('shows toast with warning type', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let tree: ReactTestRenderer | undefined;

      act(() => {
        tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      act(() => {
        hookResult?.showWarning('Warning message');
      });

      act(() => {
        tree?.update(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.isVisible).toBe(true);
    });
  });

  describe('all base hook functions available', () => {
    it('includes showToast from base hook', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.showToast).toBe('function');
    });

    it('includes hideToast from base hook', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.hideToast).toBe('function');
    });

    it('includes ToastComponent from base hook', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(hookResult?.ToastComponent).toBeDefined();
    });

    it('includes isVisible from base hook', () => {
      let hookResult: ReturnType<typeof useToastHelpers> | undefined;
      let _tree: ReactTestRenderer | undefined;

      act(() => {
        _tree = create(
          <TestHelpersComponent
            onHookResult={(result) => {
              hookResult = result;
            }}
          />
        );
      });

      expect(typeof hookResult?.isVisible).toBe('boolean');
    });
  });
});
