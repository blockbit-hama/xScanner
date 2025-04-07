use xScaner::coin::ethereum::client::EthereumClient;

#[tokio::test]
pub async fn ethreum_test() -> eyre::Result<()> {

  let block_number : u64 = 6008149;
  
  let api_url= "https://mainnet.infura.io/v3/143c82adb1704e108b6896b5b9ae9099".to_string();
  
  let response_result = EthereumClient::new(api_url).fetch_block_by_number(block_number).await;
  
  match response_result {
    Ok(response) => {
      if let Some(result) = response.result {
        println!("Successfully fetched block: {:?}", result);

        let beneficiaryList = result.transactions.iter().map(|t| {
          t.to.clone()
        });

        beneficiaryList.for_each(|to|{
          println!("to: {:?}", to);
        });

      } else if let Some(error) = response.error {
        println!("Error occurred: code = {}, message = {}", error.code, error.message);
      } else {
        println!("Unknown response format");
      }
    }
    Err(err) => {
      println!("Failed to parse JSON response: {:?}", err);
    }
  }

  Ok(())
}

