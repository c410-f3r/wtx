use crate::{
  database::client::postgres::{authentication::Authentication, DbError, PostgresError},
  misc::{bytes_rsplit1, bytes_split1, from_utf8_basic, ConnectionState, FromRadix10},
};
use core::any::type_name;

#[derive(Debug)]
pub(crate) struct Message<'bytes> {
  pub(crate) tag: u8,
  pub(crate) ty: MessageTy<'bytes>,
}

/// Messages that five bytes as well as their corresponding rest.
#[derive(Debug)]
pub(crate) enum MessageTy<'bytes> {
  /// See [Authentication].
  Authentication(Authentication<'bytes>),
  /// Data that the frontend must use to issue a cancellation request.
  BackendKeyData,
  /// Bind request was successful.
  BindComplete,
  /// Close request was successful.
  CloseComplete,
  /// Command request was successful.
  CommandComplete(u64),
  /// Data being copied using COPY.
  CopyData,
  /// COPY command finished.
  CopyDone,
  /// Starting of a COPY command from the client to the server.
  CopyInResponse,
  /// Starting of a COPY command from the server to the client.
  CopyOutResponse,
  /// Row containing the number of columns with values.
  DataRow(u16),
  /// Empty query response.
  EmptyQueryResponse,
  /// No data could be sent.
  NoData,
  /// Information response.
  NoticeResponse,
  /// Notification response.
  NotificationResponse,
  /// Parameters of a query.
  ParameterDescription(u16, &'bytes [u8]),
  /// Parameter status report.
  ParameterStatus(&'bytes [u8], &'bytes [u8]),
  /// Parse request was successful.
  ParseComplete,
  /// Row-count limit was reached.
  PortalSuspended,
  /// Backend is ready to process another query.
  ReadyForQuery,
  /// Single row data.
  RowDescription(u16, &'bytes [u8]),
}

impl<'bytes> TryFrom<(&mut ConnectionState, &'bytes [u8])> for MessageTy<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: (&mut ConnectionState, &'bytes [u8])) -> Result<Self, Self::Error> {
    let rslt = match from.1 {
      [b'1', ..] => Self::ParseComplete,
      [b'2', ..] => Self::BindComplete,
      [b'3', ..] => Self::CloseComplete,
      [b'A', ..] => Self::NotificationResponse,
      [b'C', _, _, _, _, rest @ ..] => {
        let rows = bytes_rsplit1(rest, b' ')
          .next()
          .and_then(|el| {
            if let [all_but_last @ .., _] = el {
              u64::from_radix_10(all_but_last).ok()
            } else {
              None
            }
          })
          .unwrap_or(0);
        Self::CommandComplete(rows)
      }
      [b'D', _, _, _, _, a, b, ..] => Self::DataRow(u16::from_be_bytes([*a, *b])),
      [b'E', _, _, _, _, rest @ ..] => {
        *from.0 = ConnectionState::Closed;
        return Err(DbError::try_from(from_utf8_basic(rest)?)?.into());
      }
      [b'G', ..] => Self::CopyInResponse,
      [b'H', ..] => Self::CopyOutResponse,
      [b'I', ..] => Self::EmptyQueryResponse,
      [b'K', _, _, _, _, _a, _b, _c, _d, _e, _f, _g, _h] => Self::BackendKeyData,
      [b'N', ..] => Self::NoticeResponse,
      [b'R', _, _, _, _, rest @ ..] => Self::Authentication(rest.try_into()?),
      [b'S', _, _, _, _, rest @ ..] => {
        let rslt = || {
          let mut iter = bytes_split1(rest, b'\0');
          let name = iter.next()?;
          let value = iter.next()?;
          let _ = iter.next()?;
          Some((name, value))
        };
        let (name, value) = rslt().ok_or(PostgresError::UnexpectedDatabaseMessageBytes)?;
        Self::ParameterStatus(name, value)
      }
      [b'T', _, _, _, _, a, b, rest @ ..] => {
        Self::RowDescription(u16::from_be_bytes([*a, *b]), rest)
      }
      [b'Z', _, _, _, _, _] => Self::ReadyForQuery,
      [b'c', ..] => Self::CopyDone,
      [b'd', ..] => Self::CopyData,
      [b'n', ..] => Self::NoData,
      [b's', ..] => Self::PortalSuspended,
      [b't', _, _, _, _, a, b, rest @ ..] => {
        Self::ParameterDescription(u16::from_be_bytes([*a, *b]), rest)
      }
      _ => {
        return Err(
          PostgresError::UnexpectedValueFromBytes { expected: type_name::<Self>() }.into(),
        );
      }
    };
    Ok(rslt)
  }
}
