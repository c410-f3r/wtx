use crate::http::Matcher;
use core::hint::black_box;

// Retrieved from the `matchit` project
macro_rules! routes {
  (literal) => {{
    routes!(@finish => "p1", "p2", "p3", "p4")
  }};
  (params) => {{
    routes!(@finish => "{p1}", "{p2}", "{p3}", "{p4}")
  }};
  (@finish => $p1:literal, $p2:literal, $p3:literal, $p4:literal) => {
    [
      concat!("/applications/", $p1, "/tokens/", $p2),
      concat!("/authorizations"),
      concat!("/authorizations/", $p1),
      concat!("/emojis"),
      concat!("/events"),
      concat!("/feeds"),
      concat!("/gists"),
      concat!("/gists/", $p1),
      concat!("/gists/", $p1, "/star"),
      concat!("/gitignore/templates"),
      concat!("/gitignore/templates/", $p1),
      concat!("/issues"),
      concat!("/legacy/issues/search/", $p1, "/", $p2, "/", $p3, "/", $p4),
      concat!("/legacy/repos/search/", $p1),
      concat!("/legacy/user/email/", $p1),
      concat!("/legacy/user/search/", $p1),
      concat!("/meta"),
      concat!("/networks/", $p1, "/", $p2, "/events"),
      concat!("/notifications"),
      concat!("/notifications/threads/", $p1),
      concat!("/notifications/threads/", $p1, "/subscription"),
      concat!("/orgs/", $p1),
      concat!("/orgs/", $p1, "/events"),
      concat!("/orgs/", $p1, "/issues"),
      concat!("/orgs/", $p1, "/members"),
      concat!("/orgs/", $p1, "/members/", $p2),
      concat!("/orgs/", $p1, "/public_members"),
      concat!("/orgs/", $p1, "/public_members/", $p2),
      concat!("/orgs/", $p1, "/repos"),
      concat!("/orgs/", $p1, "/teams"),
      concat!("/rate_limit"),
      concat!("/repos/", $p1, "/", $p2),
      concat!("/repos/", $p1, "/", $p2, "/assignees"),
      concat!("/repos/", $p1, "/", $p2, "/assignees/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/branches"),
      concat!("/repos/", $p1, "/", $p2, "/branches/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/collaborators"),
      concat!("/repos/", $p1, "/", $p2, "/collaborators/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/comments"),
      concat!("/repos/", $p1, "/", $p2, "/commits"),
      concat!("/repos/", $p1, "/", $p2, "/commits/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/commits/", $p3, "/comments"),
      concat!("/repos/", $p1, "/", $p2, "/contributors"),
      concat!("/repos/", $p1, "/", $p2, "/downloads"),
      concat!("/repos/", $p1, "/", $p2, "/downloads/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/events"),
      concat!("/repos/", $p1, "/", $p2, "/forks"),
      concat!("/repos/", $p1, "/", $p2, "/git/blobs/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/git/commits/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/git/refs"),
      concat!("/repos/", $p1, "/", $p2, "/git/tags/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/git/trees/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/hooks"),
      concat!("/repos/", $p1, "/", $p2, "/hooks/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/issues"),
      concat!("/repos/", $p1, "/", $p2, "/issues/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/issues/", $p3, "/comments"),
      concat!("/repos/", $p1, "/", $p2, "/issues/", $p3, "/events"),
      concat!("/repos/", $p1, "/", $p2, "/issues/", $p3, "/labels"),
      concat!("/repos/", $p1, "/", $p2, "/keys"),
      concat!("/repos/", $p1, "/", $p2, "/keys/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/labels"),
      concat!("/repos/", $p1, "/", $p2, "/labels/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/languages"),
      concat!("/repos/", $p1, "/", $p2, "/milestones/"),
      concat!("/repos/", $p1, "/", $p2, "/milestones/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/milestones/", $p3, "/labels"),
      concat!("/repos/", $p1, "/", $p2, "/notifications"),
      concat!("/repos/", $p1, "/", $p2, "/pulls"),
      concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/comments"),
      concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/commits"),
      concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/files"),
      concat!("/repos/", $p1, "/", $p2, "/pulls/", $p3, "/merge"),
      concat!("/repos/", $p1, "/", $p2, "/readme"),
      concat!("/repos/", $p1, "/", $p2, "/releases"),
      concat!("/repos/", $p1, "/", $p2, "/releases/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/releases/", $p3, "/assets"),
      concat!("/repos/", $p1, "/", $p2, "/stargazers"),
      concat!("/repos/", $p1, "/", $p2, "/stats/code_frequency"),
      concat!("/repos/", $p1, "/", $p2, "/stats/commit_activity"),
      concat!("/repos/", $p1, "/", $p2, "/stats/contributors"),
      concat!("/repos/", $p1, "/", $p2, "/stats/participation"),
      concat!("/repos/", $p1, "/", $p2, "/stats/punch_card"),
      concat!("/repos/", $p1, "/", $p2, "/statuses/", $p3),
      concat!("/repos/", $p1, "/", $p2, "/subscribers"),
      concat!("/repos/", $p1, "/", $p2, "/subscription"),
      concat!("/repos/", $p1, "/", $p2, "/tags"),
      concat!("/repos/", $p1, "/", $p2, "/teams"),
      concat!("/repositories"),
      concat!("/search/code"),
      concat!("/search/issues"),
      concat!("/search/repositories"),
      concat!("/search/users"),
      concat!("/teams/", $p1),
      concat!("/teams/", $p1, "/members"),
      concat!("/teams/", $p1, "/members/", $p2),
      concat!("/teams/", $p1, "/repos"),
      concat!("/teams/", $p1, "/repos/", $p2, "/", $p3),
      concat!("/user"),
      concat!("/user/emails"),
      concat!("/user/followers"),
      concat!("/user/following"),
      concat!("/user/following/", $p1),
      concat!("/user/issues"),
      concat!("/user/keys"),
      concat!("/user/keys/", $p1),
      concat!("/user/orgs"),
      concat!("/user/repos"),
      concat!("/user/starred"),
      concat!("/user/starred/", $p1, "/", $p2),
      concat!("/user/subscriptions"),
      concat!("/user/subscriptions/", $p1, "/", $p2),
      concat!("/user/teams"),
      concat!("/users"),
      concat!("/users/", $p1),
      concat!("/users/", $p1, "/events"),
      concat!("/users/", $p1, "/events/orgs/", $p2),
      concat!("/users/", $p1, "/events/public"),
      concat!("/users/", $p1, "/followers"),
      concat!("/users/", $p1, "/following"),
      concat!("/users/", $p1, "/following/", $p2),
      concat!("/users/", $p1, "/gists"),
      concat!("/users/", $p1, "/keys"),
      concat!("/users/", $p1, "/orgs"),
      concat!("/users/", $p1, "/received_events"),
      concat!("/users/", $p1, "/received_events/public"),
      concat!("/users/", $p1, "/repos"),
      concat!("/users/", $p1, "/starred"),
      concat!("/users/", $p1, "/subscriptions"),
    ]
  };
}

#[bench]
fn routes(b: &mut test::Bencher) {
  let routes = routes!(literal).to_vec();
  let mut matcher = Matcher::default();
  {
    let mut builder = matcher.builder();
    for route in routes!(params) {
      let _ = builder.add(route.try_into().unwrap(), true).unwrap();
    }
  }
  b.iter(|| {
    for route in black_box(&routes) {
      assert!(*black_box(matcher.find(route).unwrap()).data());
    }
  });
}
