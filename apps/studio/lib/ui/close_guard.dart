/// *Save changes to “lamp”?* — the one question every path that unloads a
/// dirty project asks (app/lifecycle.dart).
///
/// The platform's shape for a document that is about to be lost: a sheet
/// with the destructive answer on the left and apart, Cancel beside the
/// default, the default *Save* answering to Return and Cancel to Esc.  It is
/// rendered from state, so each answer is a dispatched action and the
/// reducer decides what is sent and when the project actually closes.
library;

import 'package:flutter/material.dart';
import 'package:flutter/services.dart';

import '../l10n/l10n.dart';
import '../app/actions.dart';
import '../app/state.dart';
import 'mac/controls.dart';
import 'mac/tokens.dart';
import 'system_sheets.dart' show SheetScrim;

class CloseGuardSheet extends StatelessWidget {
  const CloseGuardSheet({super.key, required this.state, required this.dispatch});
  final AppState state;
  final void Function(AppAction) dispatch;

  @override
  Widget build(BuildContext context) {
    final t = MacTokens.of(context);
    final name = state.project?.name ?? context.l10n.thisProject;
    final what = switch (state.editor.closeGuard) {
      Quit() => 'quit',
      OpenAnother() || PickAnother() => context.l10n.openAnotherProject,
      CreateAnother() || PickNew() => context.l10n.createAnotherProject,
      _ => context.l10n.closeIt,
    };
    return SheetScrim(
      title: context.l10n.saveChangesTo(name),
      width: 440,
      child: CallbackShortcuts(
        bindings: {
          const SingleActivator(LogicalKeyboardKey.escape): () =>
              dispatch(const CloseGuardAnswered(CloseGuardAnswer.cancel)),
          const SingleActivator(LogicalKeyboardKey.enter): () =>
              dispatch(const CloseGuardAnswered(CloseGuardAnswer.save)),
        },
        child: Focus(
          autofocus: true,
          child: Column(
            mainAxisSize: MainAxisSize.min,
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              Text(
                context.l10n.unsavedChangesExplanation(what),
                style: TextStyle(fontSize: 12, color: t.textSecondary),
              ),
              const SizedBox(height: 18),
              Row(
                children: [
                  MacButton(
                    label: context.l10n.donTSave,
                    onPressed: () => dispatch(const CloseGuardAnswered(CloseGuardAnswer.dontSave)),
                  ),
                  const Spacer(),
                  MacButton(
                    label: context.l10n.cancel,
                    onPressed: () => dispatch(const CloseGuardAnswered(CloseGuardAnswer.cancel)),
                  ),
                  const SizedBox(width: 8),
                  MacButton.primary(
                    label: context.l10n.save,
                    onPressed: () => dispatch(const CloseGuardAnswered(CloseGuardAnswer.save)),
                  ),
                ],
              ),
            ],
          ),
        ),
      ),
    );
  }
}
