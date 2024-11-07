## Bitcoin Explorer

### Summary
------------------------------------------------------------------------
**_This project contains RESTful Web Application that provides API endpoint to fetch Bitcoin block chain metrics and it also makes continuous automatic ingestion about block chain to the database_**

### To run this application:
------------------------------------------------------------------------

**Prerequisite:**

- [Bitcoin Core](https://bitcoin.org/en/download)
- [PostgreSQL](https://www.postgresql.org/download/)
- [Rust](https://www.rust-lang.org/)


**Useful Commands**

1. Create build:
    ```
    cargo build
    ```

2. Run applicationn:
    ```
    cargo run
    ```


### Valid Endpoints:

**_Get Metrics_**
**GET** https://{domainName}/block_metrics


**Note:** _Update `.env` file before running the application._

_This project is part of CSYE7500 course_
_Author: Souvik Dinda_
