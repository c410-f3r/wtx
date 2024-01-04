use crate::{
  database::client::postgres::{Authentication, DbError},
  misc::{_atoi, from_utf8_basic_rslt},
};
use core::any::type_name;

#[derive(Debug)]
pub(crate) struct Message<'bytes> {
  pub(crate) tag: u8,
  pub(crate) ty: MessageTy<'bytes>,
}

/// Messages that five bytes as well as their corresponding rest.
#[allow(
  // False positive
  unused_tuple_struct_fields
)]
#[derive(Debug)]
pub(crate) enum MessageTy<'bytes> {
  /// See [Authentication].
  Authentication(Authentication<'bytes>),
  /// Data that the frontend must use to issue a cancellation request.
  BackendKeyData(i32, i32),
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
  DataRow(u16, &'bytes [u8]),
  /// Empty query response.
  EmptyQueryResponse,
  /// No data could be sent.
  NoData,
  /// Information response.
  NoticeResponse,
  /// Notification response.
  NotificationResponse,
  /// Parameters of a query.
  ParameterDescription(&'bytes [u8]),
  /// Parameter status report.
  ParameterStatus(&'bytes [u8], &'bytes [u8]),
  /// Parse request was successful.
  ParseComplete,
  /// Row-count limit was reached.
  PortalSuspended,
  /// Backend is ready to process another query.
  ReadyForQuery,
  /// Single row data.
  RowDescription(&'bytes [u8]),
}

impl<'bytes> TryFrom<(&mut bool, &'bytes [u8])> for MessageTy<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(from: (&mut bool, &'bytes [u8])) -> Result<Self, Self::Error> {
    let rslt = match from.1 {
      [b'1', ..] => Self::ParseComplete,
      [b'2', ..] => Self::BindComplete,
      [b'3', ..] => Self::CloseComplete,
      [b'A', ..] => Self::NotificationResponse,
      [b'C', _, _, _, _, rest @ ..] => {
        let rows = rest
          .rsplit(|&el| el == b' ')
          .next()
          .and_then(
            |el| {
              if let [all_but_last @ .., _] = el {
                _atoi(all_but_last).ok()
              } else {
                None
              }
            },
          )
          .unwrap_or(0);
        Self::CommandComplete(rows)
      }
      [b'D', _, _, _, _, a, b, rest @ ..] => Self::DataRow(u16::from_be_bytes([*a, *b]), rest),
      [b'E', _, _, _, _, rest @ ..] => {
        *from.0 = true;
        return Err(DbError::try_from(from_utf8_basic_rslt(rest)?)?.into());
      }
      [b'G', ..] => Self::CopyInResponse,
      [b'H', ..] => Self::CopyOutResponse,
      [b'I', ..] => Self::EmptyQueryResponse,
      [b'K', _, _, _, _, a, b, c, d, e, f, g, h] => Self::BackendKeyData(
        i32::from_be_bytes([*a, *b, *c, *d]),
        i32::from_be_bytes([*e, *f, *g, *h]),
      ),
      [b'N', ..] => Self::NoticeResponse,
      [b'R', _, _, _, _, rest @ ..] => Self::Authentication(rest.try_into()?),
      [b'S', _, _, _, _, rest @ ..] => {
        let rslt = || {
          let mut iter = rest.split(|elem| elem == &0);
          let name = iter.next()?;
          let value = iter.next()?;
          let _ = iter.next()?;
          Some((name, value))
        };
        let (name, value) = rslt().ok_or(crate::Error::UnexpectedDatabaseMessageBytes)?;
        Self::ParameterStatus(name, value)
      }
      [b'T', _, _, _, _, _a, _b, rest @ ..] => Self::RowDescription(rest),
      [b'Z', _, _, _, _, _] => Self::ReadyForQuery,
      [b'c', ..] => Self::CopyDone,
      [b'd', ..] => Self::CopyData,
      [b'n', ..] => Self::NoData,
      [b's', ..] => Self::PortalSuspended,
      [b't', _, _, _, _, _a, _b, rest @ ..] => Self::ParameterDescription(rest),
      _ => return Err(crate::Error::UnexpectedValueFromBytes { expected: type_name::<Self>() }),
    };
    Ok(rslt)
  }
}
