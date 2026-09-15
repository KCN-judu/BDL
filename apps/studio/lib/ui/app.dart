import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../app/actions.dart';
import '../app/store.dart';
import 'home_page.dart';

class StudioApp extends ConsumerStatefulWidget {
  const StudioApp({super.key});

  @override
  ConsumerState<StudioApp> createState() => _StudioAppState();
}

class _StudioAppState extends ConsumerState<StudioApp> {
  @override
  void initState() {
    super.initState();
    // The app's first action; everything after is state transitions.
    WidgetsBinding.instance.addPostFrameCallback((_) {
      ref.read(appStoreProvider.notifier).dispatch(const AppStarted());
    });
  }

  @override
  Widget build(BuildContext context) {
    return MaterialApp(
      title: 'BDL Studio',
      theme: ThemeData(colorSchemeSeed: const Color(0xFF3B6EA5), useMaterial3: true),
      darkTheme: ThemeData(
        colorSchemeSeed: const Color(0xFF3B6EA5),
        brightness: Brightness.dark,
        useMaterial3: true,
      ),
      home: const HomePage(),
    );
  }
}
