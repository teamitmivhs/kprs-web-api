use std::{collections::HashMap, fs::File, io::{BufRead, BufReader}};
use once_cell::sync::Lazy;


pub static USERS_DATA: Lazy<HashMap<String, String>> = Lazy::new(|| {
      // Create a variable that can hold the data
      let mut users_data: HashMap<String, String> = HashMap::new();

      // Get the file stream from user.txt
      let users_data_file = File::open("user.txt").unwrap();

      // Get the buffer reader
      let reader = BufReader::new(users_data_file);

      // Iterate for each line
      for line in reader.lines() {
            let line = line.unwrap();                                                         // Panic if there's an error reading the file
            let splitted_text = line.splitn(2, "_").collect::<Vec<&str>>();         // Split line by underscore (_)
            let [username, token] = [                                                 // Get the username and token for each line
                  splitted_text.first().unwrap().to_string(),
                  splitted_text.last().unwrap().to_string()
            ];
            users_data.insert(token, username);                                                  // Push the username and token to the users data
      }

      // Return the result
      return users_data;
});