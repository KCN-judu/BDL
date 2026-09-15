import 'gen/bdl/v1/bdl.pb.dart' as pb;

/// Protocol version this client speaks.  Must share `major` with the daemon.
final pb.Version kClientProtocolVersion = pb.Version(major: 0, minor: 3, patch: 0)..freeze();

const String kStudioVersion = '0.1.0';
const String kClientName = 'bdl-studio';

String formatVersion(pb.Version v) => '${v.major}.${v.minor}.${v.patch}';
