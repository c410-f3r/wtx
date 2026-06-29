use crate::{
  codec::FromRadix10 as _,
  collections::ShortStrU8,
  database::{
    DatabaseError,
    client::postgres::{DbError, PostgresError, authentication::Authentication},
  },
  misc::{ConnectionState, bytes_rsplit1, bytes_split1, from_utf8_basic},
};
use core::any::type_name;

#[derive(Debug)]
pub(crate) struct Message<'bytes> {
  pub(crate) tag: u8,
  pub(crate) ty: MessageTy<'bytes>,
}

/// Messages that five bytes as well as their corresponding data.
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
  CommandComplete(u32),
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
  ParameterDescription(u16),
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

impl<'bytes> TryFrom<(&mut ConnectionState, u8, &'bytes [u8])> for MessageTy<'bytes> {
  type Error = crate::Error;

  #[inline]
  fn try_from(
    (cs, tag, payload): (&mut ConnectionState, u8, &'bytes [u8]),
  ) -> Result<Self, Self::Error> {
    let rslt = match (tag, payload) {
      (b'1', _) => Self::ParseComplete,
      (b'2', _) => Self::BindComplete,
      (b'3', _) => Self::CloseComplete,
      (b'A', _) => Self::NotificationResponse,
      (b'C', data) => {
        let rows = bytes_rsplit1(data, b' ')
          .next()
          .and_then(|el| {
            if let [all_but_last @ .., _] = el {
              u32::from_radix_10(all_but_last).ok()
            } else {
              None
            }
          })
          .unwrap_or(0);
        Self::CommandComplete(rows)
      }
      (b'D', [b0, b1, ..]) => Self::DataRow(u16::from_be_bytes([*b0, *b1])),
      (b'E', data) => {
        *cs = ConnectionState::Closed;
        return Err(DbError::try_from(from_utf8_basic(data)?)?.into());
      }
      (b'G', _) => Self::CopyInResponse,
      (b'H', _) => Self::CopyOutResponse,
      (b'I', _) => Self::EmptyQueryResponse,
      (b'K', [_, _, _, _, _, _, _, _]) => Self::BackendKeyData,
      (b'N', _) => Self::NoticeResponse,
      (b'R', data) => Self::Authentication(data.try_into()?),
      (b'S', data) => {
        let rslt = || {
          let mut iter = bytes_split1(data, b'\0');
          let name = iter.next()?;
          let value = iter.next()?;
          let _ = iter.next()?;
          Some((name, value))
        };
        let (name, value) = rslt().ok_or(PostgresError::UnexpectedDatabaseMessageBytes)?;
        Self::ParameterStatus(name, value)
      }
      (b'T', [b0, b1, data @ ..]) => Self::RowDescription(u16::from_be_bytes([*b0, *b1]), data),
      (b'Z', _) => Self::ReadyForQuery,
      (b'c', _) => Self::CopyDone,
      (b'd', _) => Self::CopyData,
      (b'n', _) => Self::NoData,
      (b's', _) => Self::PortalSuspended,
      (b't', [b0, b1, ..]) => Self::ParameterDescription(u16::from_be_bytes([*b0, *b1])),
      _ => {
        return Err(
          DatabaseError::UnexpectedValueFromBytes {
            expected: ShortStrU8::new_truncated_u8(type_name::<Self>()),
          }
          .into(),
        );
      }
    };
    Ok(rslt)
  }
}
