use crate::app_state::AppState;
use csv::Reader;
use std::sync::Arc;
use chrono::NaiveDate;

pub async fn run_load(app_state: Arc<AppState>) {
    let client = app_state.pool.get().await.unwrap();
    let statement = client.prepare_cached("INSERT INTO users (first_name, second_name, birthdate, city, pwd) VALUES ($1, $2, $3, $4, $5)").await.unwrap();            
    let mut rdr = Reader::from_path("data/people.v2.csv").unwrap();
    
    for result in rdr.records() {
        let record = result.unwrap();        
        let mut name = record.get(0).unwrap().split_whitespace();
        let first = name.next();
        let last = name.next();
        let date = NaiveDate::parse_from_str(record.get(1).unwrap(), "%Y-%m-%d").unwrap();
        println!("{:?} {:?} {:?} {:?} {:?}", 
            first, 
            last, 
            date, 
            record.get(2),
            client.execute(
                &statement, 
                &[
                    &first.unwrap(),
                    &last.unwrap(),
                    &date,
                    &record.get(2).unwrap(),
                    &"$argon2id$v=19$m=19456,t=2,p=1$ceguLp4Epue7eUDreYr16A$2jdgGIzB17jtFnlnZ/4wmkjbz/Avt5cea7Y9YYxH5sk"
                ]).await
        );
    }
}