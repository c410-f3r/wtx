use crate::collections::ShortStrU8;

/// Used to specify the data type that is going to be sent to a counterpart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mime {
  /// application/grpc
  ApplicationGrpc,
  /// application/json
  ApplicationJson,
  /// application/octet-stream
  ApplicationOctetStream,
  /// application/pdf
  ApplicationPdf,
  /// application/vnd.google.protobuf
  ApplicationVndGoogleProtobuf,
  /// application/wasm
  ApplicationWasm,
  /// application/xml
  ApplicationXml,
  /// application/x-www-form-urlencoded
  ApplicationXWwwFormUrlEncoded,
  /// application/yaml
  ApplicationYaml,
  /// application/zip
  ApplicationZip,
  /// audio/mpeg
  AudioMpeg,
  /// audio/ogg
  AudioOgg,
  /// audio/webm
  AudioWebm,
  /// Anything
  Custom(ShortStrU8<'static>),
  /// font/woff
  FontWoff,
  /// font/woff2
  FontWoff2,
  /// image/avif
  ImageAvif,
  /// image/gif
  ImageGif,
  /// image/jpeg
  ImageJpeg,
  /// image/png
  ImagePng,
  /// image/svg+xml
  ImageSvgXml,
  /// image/webp
  ImageWebp,
  /// image/x-icon
  ImageXIcon,
  /// multipart/form-data
  MultipartFormData,
  /// text/css
  TextCss,
  /// text/csv
  TextCsv,
  /// text/html
  TextHtml,
  /// text/javascript
  TextJavascript,
  /// text/markdown
  TextMarkdown,
  /// text/plain
  TextPlain,
  /// video/mp4
  VideoMp4,
  /// video/mpeg
  VideoMpeg,
  /// video/webm
  VideoWebm,
}

impl Mime {
  /// Common string representation.
  #[inline]
  pub fn as_str(&self) -> &'static str {
    match self {
      Self::ApplicationGrpc => "application/grpc",
      Self::ApplicationJson => "application/json",
      Self::ApplicationOctetStream => "application/octet-stream",
      Self::ApplicationPdf => "application/pdf",
      Self::ApplicationVndGoogleProtobuf => "application/vnd.google.protobuf",
      Self::ApplicationWasm => "application/wasm",
      Self::ApplicationXml => "application/xml",
      Self::ApplicationXWwwFormUrlEncoded => "application/x-www-form-urlencoded",
      Self::ApplicationYaml => "application/yaml",
      Self::ApplicationZip => "application/zip",
      Self::AudioMpeg => "audio/mpeg",
      Self::AudioOgg => "audio/ogg",
      Self::AudioWebm => "audio/webm",
      Self::Custom(el) => el.into_str(),
      Self::FontWoff => "font/woff",
      Self::FontWoff2 => "font/woff2",
      Self::ImageAvif => "image/avif",
      Self::ImageGif => "image/gif",
      Self::ImageJpeg => "image/jpeg",
      Self::ImagePng => "image/png",
      Self::ImageSvgXml => "image/svg+xml",
      Self::ImageWebp => "image/webp",
      Self::ImageXIcon => "image/x-icon",
      Self::MultipartFormData => "multipart/form-data",
      Self::TextCss => "text/css",
      Self::TextCsv => "text/csv",
      Self::TextHtml => "text/html",
      Self::TextJavascript => "text/javascript",
      Self::TextMarkdown => "text/markdown",
      Self::TextPlain => "text/plain",
      Self::VideoMp4 => "video/mp4",
      Self::VideoMpeg => "video/mpeg",
      Self::VideoWebm => "video/webm",
    }
  }
}
