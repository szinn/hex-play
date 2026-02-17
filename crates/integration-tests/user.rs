use hex_play_core::models::user::{NewUser, PartialUserUpdate};

use crate::postgres::setup;

#[tokio::test]
async fn test_user_lifecycle() {
    let (_container, svc) = setup().await;

    // 1. Create user
    let new_user = NewUser::new("Alice", "alice@test.com", 28).unwrap();
    let created = svc.user_service.add_user(new_user).await.unwrap();

    let token = created.token;
    let created_version = created.version;
    let created_updated_at = created.updated_at;

    assert_eq!(created.name, "Alice");
    assert_eq!(created.email.to_string(), "alice@test.com");
    assert_eq!(created.age.value(), 28);
    assert_eq!(created_version, 1);

    // 2. Fetch by token
    let fetched = svc.user_service.find_by_token(token).await.unwrap().unwrap();

    assert_eq!(fetched.name, "Alice");
    assert_eq!(fetched.email.to_string(), "alice@test.com");
    assert_eq!(fetched.age.value(), 28);
    assert_eq!(fetched.version, created_version);
    assert_eq!(fetched.updated_at, created_updated_at);

    // 3. Update email
    let mut user_to_update = fetched;
    let update = PartialUserUpdate::new(None::<String>, Some("alice.updated@test.com"), None).unwrap();
    update.apply_to(&mut user_to_update);

    let updated = svc.user_service.update_user(user_to_update).await.unwrap();

    assert_eq!(updated.email.to_string(), "alice.updated@test.com");
    assert!(updated.version > created_version);
    assert!(updated.updated_at >= created_updated_at);

    // 4. Verify update persisted
    let verified = svc.user_service.find_by_token(token).await.unwrap().unwrap();

    assert_eq!(verified.email.to_string(), "alice.updated@test.com");
    assert_eq!(verified.version, updated.version);
    assert_eq!(verified.updated_at, updated.updated_at);
}
