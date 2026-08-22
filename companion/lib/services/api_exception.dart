/// Thrown by [ApiClient] for any non-2xx REST response, or a response body
/// that couldn't be decoded as expected.
class ApiException implements Exception {
  final String message;
  final int? statusCode;

  const ApiException(this.message, {this.statusCode});

  @override
  String toString() =>
      statusCode != null ? 'ApiException($statusCode): $message' : 'ApiException: $message';
}
