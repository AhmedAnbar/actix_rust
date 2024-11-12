use serde::Serialize;
use std::error::Error;

// Function to export data to CSV format
pub fn export_to_csv<T: Serialize>(data: &[T]) -> Result<String, Box<dyn Error>> {
    // Create a CSV writer that writes to a vector
    let mut wtr = csv::Writer::from_writer(vec![]);

    if data.is_empty() {
        // Manually write the header if data is empty
        wtr.write_record(&["field1", "field2"])?;
    } else {
        // Serialize each record and write to CSV
        for record in data {
            wtr.serialize(record)?; // Serialize the record and handle any serialization errors
        }
    }

    // Retrieve the CSV data as a UTF-8 string
    let csv_data = String::from_utf8(wtr.into_inner()?)?;

    // Return the CSV data
    Ok(csv_data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Serialize;

    #[derive(Serialize)]
    struct TestData {
        field1: String,
        field2: i32,
    }

    #[test]
    fn test_export_to_csv() {
        // Prepare the test data
        let data = vec![
            TestData {
                field1: "value1".to_string(),
                field2: 1,
            },
            TestData {
                field1: "value2".to_string(),
                field2: 2,
            },
        ];

        // Call the export_to_csv function
        let result = export_to_csv(&data);

        // Ensure the result is Ok
        assert!(result.is_ok());

        // Get the CSV data from the result
        let csv_data = result.unwrap();

        // Expected CSV output
        let expected_csv = "field1,field2\nvalue1,1\nvalue2,2\n";

        // Assert that the CSV data matches the expected output
        assert_eq!(csv_data, expected_csv);
    }

    #[test]
    fn test_export_to_csv_empty_data() {
        // Prepare empty test data
        let data: Vec<TestData> = vec![];

        // Call the export_to_csv function
        let result = export_to_csv(&data);

        // Ensure the result is Ok
        assert!(result.is_ok());

        // Get the CSV data from the result
        let csv_data = result.unwrap();

        // Expected CSV output for empty data (only the header)
        let expected_csv = "field1,field2\n";

        // Assert that the CSV data matches the expected output
        assert_eq!(csv_data, expected_csv);
    }
}

