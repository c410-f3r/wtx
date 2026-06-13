use crate::{
  collection::ArrayVectorU8,
  http::{
    Router, RouterMatchParam,
    router::{Edge, Row, RowTy},
  },
};

#[test]
fn add_static_route_after_param_route_and_vice_versa() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/user/static".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/user/1").unwrap().data(), &1);
  assert_eq!(matcher.find("/user/static").unwrap().data(), &2);

  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/static".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/user/{id}".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/user/static").unwrap().data(), &1);
  assert_eq!(matcher.find("/user/1").unwrap().data(), &2);
}

#[test]
fn deep_traversal_after_splits() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/api/users".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/api/posts".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/api/comments".try_into().unwrap(), 3).unwrap();
  }

  assert_eq!(matcher.find("/api/users").unwrap().data(), &1);
  assert_eq!(matcher.find("/api/posts").unwrap().data(), &2);
  assert_eq!(matcher.find("/api/comments").unwrap().data(), &3);
  assert!(matcher.find("/api").is_err());
  assert!(matcher.find("/api/user").is_err());
}

#[test]
fn duplicated_route() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/abcd".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/ab".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/abxy".try_into().unwrap(), 3).unwrap();
    assert!(builder.add(&"/ab".try_into().unwrap(), 4).is_err());
  }

  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}".try_into().unwrap(), 1).unwrap();
    assert!(builder.add(&"/user/{name}".try_into().unwrap(), 2).is_err());
  }
}

#[test]
fn empty_route() {
  let mut matcher = Router::default();
  let mut builder = matcher.builder();
  assert!(builder.add(&"".try_into().unwrap(), 1).is_err());
}

#[test]
fn find_on_intermediate_node_without_value() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/foo/bar".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/foo/baz".try_into().unwrap(), 2).unwrap();
  }
  assert!(matcher.find("/foo/").is_err());
}

#[test]
fn lesser_ident_in_intermediate_node() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/foo/bar".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/foo/baz".try_into().unwrap(), 2).unwrap();
  }
  let path = matcher.find("/foo/b");
  assert!(path.is_err());
}

// ```
// / -> aaa -> /bbb
//    \      \
//     \      \-> /ccc
//      \
//       \-> /ddd
// ```
//
// ```
// /
// aaa/
// ddd
// ccc
// bbb
// ```
#[test]
fn literal() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/aaa/bbb".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/aaa/ccc".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/ddd".try_into().unwrap(), 3).unwrap();
  }
  assert_eq!(
    &matcher.rows,
    &[
      Row::new(
        ArrayVectorU8::from_iterator([Edge::new(b'd'.into(), 4), Edge::new(b'a'.into(), 3)])
          .unwrap(),
        "/".try_into().unwrap(),
        RowTy::Literal,
        None
      ),
      Row::new(ArrayVectorU8::new(), "bbb".try_into().unwrap(), RowTy::Literal, Some(1)),
      Row::new(ArrayVectorU8::new(), "ccc".try_into().unwrap(), RowTy::Literal, Some(2)),
      Row::new(
        ArrayVectorU8::from_iterator([Edge::new(b'b'.into(), 1), Edge::new(b'c'.into(), 2)])
          .unwrap(),
        "aaa/".try_into().unwrap(),
        RowTy::Literal,
        None
      ),
      Row::new(ArrayVectorU8::new(), "ddd".try_into().unwrap(), RowTy::Literal, Some(3)),
    ]
  );
  assert_eq!(matcher.find("/aaa/bbb").unwrap().data(), &1);
  assert_eq!(matcher.find("/aaa/ccc").unwrap().data(), &2);
  assert_eq!(matcher.find("/ddd").unwrap().data(), &3);
  assert!(matcher.find("").is_err());
  assert!(matcher.find("/aaa").is_err());
  assert!(matcher.find("/aaa/bb").is_err());
  assert!(matcher.find("/aaa/bbbb").is_err());
  assert!(matcher.find("/eee").is_err());
}

#[test]
fn multiple_root_nodes_edge_tracking() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/foo".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/bar".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/baz".try_into().unwrap(), 3).unwrap();
  }
  assert_eq!(matcher.find("/foo").unwrap().data(), &1);
  assert_eq!(matcher.find("/bar").unwrap().data(), &2);
  assert_eq!(matcher.find("/baz").unwrap().data(), &3);
}

#[test]
fn multiple_sub_routes() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/a/b/c".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/a/b/d".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/a/{any}/e".try_into().unwrap(), 3).unwrap();
  }
  assert_eq!(matcher.find("/a/b/c").unwrap().data(), &1);
  assert_eq!(matcher.find("/a/b/d").unwrap().data(), &2);
  assert!(matcher.find("/a/b/e").is_err()); // No backtrack
  assert_eq!(matcher.find("/a/foo/e").unwrap().data(), &3);
}

#[test]
fn nested_paths() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/a".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/a/b".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/a/b/c".try_into().unwrap(), 3).unwrap();
    let _ = builder.add(&"/a/b/c/d".try_into().unwrap(), 4).unwrap();
    let _ = builder.add(&"/a/b/c/d/e".try_into().unwrap(), 5).unwrap();
    let _ = builder.add(&"/a/b/c/d/e/f".try_into().unwrap(), 6).unwrap();
    let _ = builder.add(&"/a/b/c/d/e/f/g".try_into().unwrap(), 7).unwrap();
    let _ = builder.add(&"/a/b/c/d/e/f/g/h".try_into().unwrap(), 8).unwrap();
    let _ = builder.add(&"/a/b/c/d/e/f/g/h/i".try_into().unwrap(), 9).unwrap();
  }
  let path = matcher.find("/a/b/c/d/e/f/g/h/i").unwrap();
  assert_eq!(*path.data(), 9);
}

#[test]
fn parameter_after() {
  let mut matcher = Router::default();
  let mut builder = matcher.builder();
  assert!(builder.add(&"/user/{id}profile".try_into().unwrap(), 1).is_err());
}

#[test]
fn parameter_after_literal() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/aaaa".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/aaaa/{bbb}".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/aaaa/rrr").unwrap().data(), &2);
}

#[test]
fn parameter_in_split_node_is_correctly_extracted() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/api/v1/user".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/api/{version}/data".try_into().unwrap(), 2).unwrap();
  }
  let path = matcher.find("/api/v2/data").unwrap();
  assert_eq!(path.data(), &2);
  assert_eq!(path.param_by_name(b"version").unwrap(), RouterMatchParam::new("version", "v2"));
}

#[test]
fn parameter_length_overflow() {
  macro_rules! long_param {
    () => {
      "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
    };
  }

  let mut matcher = Router::default();
  let mut builder = matcher.builder();
  let Ok(route) = concat!("/user/", "{", long_param!(), "}").try_into() else {
    return;
  };
  // Future-proof
  assert!(builder.add(&route, 1).is_err());
}

#[test]
fn parameter_multiple_children_intermediate() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/a/w{p}".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/a/x{p}".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/a/y{p}".try_into().unwrap(), 3).unwrap();
  }
  let path = matcher.find("/a/y123");
  assert_eq!(path.unwrap().data(), &3);
}

#[test]
fn parameter_name_loss() {
  let mut matcher = Router::<i32>::new();
  let mut builder = matcher.builder();
  let _ = builder.add(&"/api/{first_name}/c".try_into().unwrap(), 1).unwrap();
  assert!(builder.add(&"/api/{second_name}/d".try_into().unwrap(), 2).is_err());
}

#[test]
fn parameter_node_split() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}/profile".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/user/{id}/settings".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/user/123/profile").unwrap().data(), &1);
  assert_eq!(matcher.find("/user/456/settings").unwrap().data(), &2);
}

#[test]
fn parameter_node_split_reverse_order() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}/profile".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/user/".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/user/").unwrap().data(), &2);
  assert_eq!(matcher.find("/user/123/profile").unwrap().data(), &1);
}

#[test]
fn parameter_with_after_split_order() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/api/{version}/users".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/api/{version}/posts".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/api/v1/users").unwrap().data(), &1);
  assert_eq!(matcher.find("/api/v1/posts").unwrap().data(), &2);
  assert!(matcher.find("/api/v1").is_err());
  assert!(matcher.find("/api/v1/other").is_err());
}

#[test]
fn parameter_without_after_then_with_slash() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/user/{id}/".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/user/123").unwrap().data(), &1);
  assert_eq!(matcher.find("/user/456/").unwrap().data(), &2);
}

#[test]
fn parameters() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/aaa/{}/bbb".try_into().unwrap(), 1).unwrap();
  }
  let path = matcher.find("/aaa/123/bbb").unwrap();
  assert_eq!(path.data(), &1);
  assert_eq!(path.param_by_idx(0).unwrap(), RouterMatchParam::new("", "123"));
  assert_eq!(path.param_by_idx(1), None);
}

#[test]
fn path_rows_corruption() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/a/w{p}".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/a/x{p}".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/a/y123".try_into().unwrap(), 3).unwrap();
  }
  let path = matcher.find("/a/y123").unwrap();
  assert_eq!(path.data(), &3);
  assert_eq!(path.params().count(), 0);
}

#[test]
fn path_rows_overflow() {
  let mut matcher = Router::<i32, 5, 2>::new();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/{a}/{b}/{c}".try_into().unwrap(), 1).unwrap();
  }
  assert!(matcher.find("/1/2/3").is_err());
}

#[test]
fn root_param_with_children() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/{id}".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/{id}/profile".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/123").unwrap().data(), &1);
  assert_eq!(matcher.find("/456/profile").unwrap().data(), &2);
}

#[test]
fn route_with_param_suffix_should_not_match_without_it() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}/profile".try_into().unwrap(), 1).unwrap();
  }
  assert!(matcher.find("/user/123").is_err());
}

#[test]
fn row_value_in_intermediary() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/foo".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/foo/bar".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/foo/bar/baz".try_into().unwrap(), 3).unwrap();
  }
  assert_eq!(matcher.find("/foo").unwrap().data(), &1);
  assert_eq!(matcher.find("/foo/bar").unwrap().data(), &2);
  assert_eq!(matcher.find("/foo/bar/baz").unwrap().data(), &3);
}

#[test]
fn several_parameters() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/aa".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/aa/{}".try_into().unwrap(), 2).unwrap();
    let _ = builder.add(&"/bb/{}".try_into().unwrap(), 3).unwrap();
    let _ = builder.add(&"/bb/{}/cc/{}".try_into().unwrap(), 4).unwrap();
  }
  assert_eq!(matcher.find("/aa").unwrap().data(), &1);
  assert_eq!(matcher.find("/aa/111").unwrap().data(), &2);
  assert_eq!(matcher.find("/bb/222").unwrap().data(), &3);
  assert_eq!(matcher.find("/bb/333/cc/444").unwrap().data(), &4);
}

#[test]
fn shorter_route_after_longer_route() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/api/users/profile".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/api/users".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/api/users/profile").unwrap().data(), &1);
  assert_eq!(matcher.find("/api/users").unwrap().data(), &2);
}

#[test]
fn single_parameter_route_should_not_match_extra_segments() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/user/{id}".try_into().unwrap(), 1).unwrap();
  }
  assert_eq!(matcher.find("/user/123").unwrap().data(), &1);
  assert!(matcher.find("/user/123/extra").is_err());
}

#[test]
fn split_node_single_wrong_identifier() {
  {
    let mut matcher = Router::default();
    {
      let mut builder = matcher.builder();
      let _ = builder.add(&"/ab".try_into().unwrap(), 1).unwrap();
      let _ = builder.add(&"/abc".try_into().unwrap(), 2).unwrap();
    }
    assert_eq!(matcher.find("/ab").unwrap().data(), &1);
    assert_eq!(matcher.find("/abc").unwrap().data(), &2);
  }

  {
    let mut matcher = Router::default();
    {
      let mut builder = matcher.builder();
      let _ = builder.add(&"/abc".try_into().unwrap(), 2).unwrap();
      let _ = builder.add(&"/ab".try_into().unwrap(), 1).unwrap();
    }
    assert_eq!(matcher.find("/ab").unwrap().data(), &1);
    assert_eq!(matcher.find("/abc").unwrap().data(), &2);
  }
}

#[test]
fn static_mixed_with_dynamic() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/api/{version}/users".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/api/v1/users".try_into().unwrap(), 2).unwrap();
  }
  assert_eq!(matcher.find("/api/1/users").unwrap().data(), &1);
  assert_eq!(matcher.find("/api/v1/users").unwrap().data(), &2);
}

#[test]
fn two_parameters_in_a_path() {
  let mut matcher = Router::default();
  {
    let mut builder = matcher.builder();
    let _ = builder.add(&"/aa/{}".try_into().unwrap(), 1).unwrap();
    let _ = builder.add(&"/aa/{}/bb/{}".try_into().unwrap(), 2).unwrap();
  }
  let path = matcher.find("/aa/111/bb/222").unwrap();
  assert_eq!(path.data(), &2);
  assert_eq!(path.param_by_idx(0).unwrap(), RouterMatchParam::new("", "111"));
  assert_eq!(path.param_by_idx(1).unwrap(), RouterMatchParam::new("", "222"));
}

#[test]
fn wrong_paths() {
  let mut matcher = Router::default();
  let mut builder = matcher.builder();
  assert!(builder.add(&"foo".try_into().unwrap(), 1).is_err());
  assert!(builder.add(&"bar".try_into().unwrap(), 2).is_err());
}
