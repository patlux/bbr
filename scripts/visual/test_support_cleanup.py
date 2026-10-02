"""Darwin zombie-group uncertainty is not cleanup proof or a reason to signal again."""
import signal
import subprocess
import unittest
from unittest.mock import Mock, call, patch

from support import terminate_group


class CleanupContracts(unittest.TestCase):
    def test_transient_eperm_requires_observed_absence_and_leader_reaping(self):
        child = Mock(pid=1234)
        with patch('support.os.killpg', side_effect=[None, PermissionError(), ProcessLookupError()]) as kill, \
             patch('support.time.sleep'):
            terminate_group(child, 3)
        self.assertEqual(kill.call_args_list, [call(1234, signal.SIGTERM), call(1234, 0), call(1234, 0)])
        self.assertGreaterEqual(child.poll.call_count, 2)
        child.wait.assert_called_once()
        self.assertLessEqual(child.wait.call_args.kwargs['timeout'], 3)

    def test_persistent_eperm_fails_closed_without_signaling_uncertain_group(self):
        child = Mock(pid=1234)
        with patch('support.os.killpg', side_effect=[None, PermissionError(), PermissionError()]) as kill, \
             patch('support.time.monotonic', side_effect=[0, 0, 0, 0, 4]), patch('support.time.sleep'):
            with self.assertRaisesRegex(RuntimeError, 'cleanup.*unproven'):
                terminate_group(child, 3)
        self.assertEqual(kill.call_args_list, [call(1234, signal.SIGTERM), call(1234, 0), call(1234, 0)])
        child.wait.assert_not_called()

    def test_live_group_after_eperm_still_requires_absence_not_another_signal(self):
        child = Mock(pid=1234)
        with patch('support.os.killpg', side_effect=[None, PermissionError(), None, ProcessLookupError()]) as kill, \
             patch('support.time.sleep'):
            terminate_group(child, 3)
        self.assertEqual(kill.call_args_list, [call(1234, signal.SIGTERM), *[call(1234, 0)] * 3])
        child.wait.assert_called_once()

    def test_unreaped_leader_cannot_be_reported_clean_after_group_absence(self):
        child = Mock(pid=1234)
        child.wait.side_effect = subprocess.TimeoutExpired('fixture', 1)
        with patch('support.os.killpg', side_effect=[None, PermissionError(), ProcessLookupError()]), \
             patch('support.time.sleep'):
            with self.assertRaises(subprocess.TimeoutExpired):
                terminate_group(child, 3)


if __name__ == '__main__':
    unittest.main()
