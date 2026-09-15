import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../app/actions.dart';
import '../app/state.dart';
import '../app/store.dart';
import '../protocol/versions.dart';

/// Compiler version, protocol version, connection state.  Presentation only.
class StatusBar extends ConsumerWidget {
  const StatusBar({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final state = ref.watch(appStoreProvider);
    final theme = Theme.of(context);
    final conn = state.connection;

    final (Color color, String label) = switch (conn) {
      Disconnected() => (theme.colorScheme.outline, 'disconnected'),
      Connecting() => (theme.colorScheme.tertiary, 'connecting…'),
      Connected() => (Colors.green, 'connected'),
      ConnectionFailed(:final reason) => (theme.colorScheme.error, 'failed: $reason'),
    };
    final compiler = switch (conn) {
      Connected(:final handshake) => handshake.compilerVersion,
      _ => '—',
    };
    final protocol = switch (conn) {
      Connected(:final handshake) => formatVersion(handshake.protocolVersion),
      _ => '—',
    };

    return Material(
      color: theme.colorScheme.surfaceContainerHighest,
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
        child: Row(
          children: [
            Icon(Icons.circle, size: 10, color: color),
            const SizedBox(width: 6),
            Flexible(
              child: Text(
                'bdld $label',
                style: theme.textTheme.bodySmall,
                overflow: TextOverflow.ellipsis,
              ),
            ),
            const SizedBox(width: 24),
            Text('compiler $compiler', style: theme.textTheme.bodySmall),
            const SizedBox(width: 24),
            Text(
              'protocol $protocol (studio ${formatVersion(kClientProtocolVersion)})',
              style: theme.textTheme.bodySmall,
            ),
            const Spacer(),
            if (state.editor.pendingRequests > 0)
              const SizedBox(
                width: 12,
                height: 12,
                child: CircularProgressIndicator(strokeWidth: 2),
              ),
            if (conn is ConnectionFailed || conn is Disconnected)
              TextButton(
                onPressed: () =>
                    ref.read(appStoreProvider.notifier).dispatch(const ConnectRequested()),
                child: const Text('Reconnect'),
              ),
          ],
        ),
      ),
    );
  }
}
