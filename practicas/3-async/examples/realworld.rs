use std::time::Duration;

use async_std::task;
use std::collections::{HashSet, HashMap};
use futures::join;
use futures::future::join_all;

struct Sale {
    id: i32,
    user_id: i32,
    product_id: i32,
    // other stuff
}

struct Product {
    id: i32,
    name: String
}

struct User {
    id: i32,
    email: String
}

#[derive(Debug)]
struct SaleView {
    id: i32,
    product: String,
    user: String
}

async fn query_db() -> Vec<Sale> {
    // DB is so slow!!!! 😱
    task::sleep(Duration::from_secs(1)).await;
    vec!(
        Sale { id: 1, user_id: 1, product_id: 1 },
        Sale { id: 2, user_id: 1, product_id: 2 },
        Sale { id: 3, user_id: 2, product_id: 3 },
    )
}

async fn find_users_by_ids(ids:HashSet<i32>) -> Vec<User> {
    // "Talks" to the user service
    task::sleep(Duration::from_secs(2)).await;
    vec!(
        User { id: 1, email: String::from("user1@user.com")},
        User { id: 2, email: String::from("user2@user.com")},
        User { id: 3, email: String::from("ignored")}
    ).into_iter().filter(|u| ids.contains(&u.id)).collect()
}

async fn find_products_by_ids(ids:HashSet<i32>) -> Vec<Product> {
    // Those 🤬 backend guys didn't make a bulk API
    let products_futures = ids.into_iter()
        .map(|id| find_product_by_id(id));
    join_all(products_futures).await
}

async fn find_product_by_id(id:i32) -> Product {
    task::sleep(Duration::from_millis(300)).await;
    Product { id, name: String::from("Product ") + id.to_string().as_str()}
}


async fn async_main() -> Vec<SaleView> {

    let sales = query_db().await;
    let user_ids = sales.iter().map(|u| u.user_id).collect();
    let product_ids = sales.iter().map(|u| u.product_id).collect();

    let users_future = find_users_by_ids(user_ids);
    let products_future = find_products_by_ids(product_ids);

    let (users, products) = join!(users_future, products_future);

    let users_by_id: HashMap<i32, &User> = users.iter().map(|u| (u.id, u)).collect();
    let products_by_id: HashMap<i32, &Product> = products.iter().map(|p| (p.id, p)).collect();

    sales.iter().map(|s| {
        let p = products_by_id.get(&s.product_id).unwrap();
        let u = users_by_id.get(&s.user_id).unwrap();

        SaleView {
            id: s.id,
            product: p.name.clone(),
            user: u.email.clone()
        }}).collect()
}

fn main() {
    println!("{:?}", task::block_on(async_main()))
}
