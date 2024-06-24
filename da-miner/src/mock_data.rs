use rand::{thread_rng, Rng};
use storage::slice_db::SliceDB;
use zg_encoder::{EncodedBlob, RawBlob, RawData, ZgEncoderParams};

pub async fn store_mock_data(param_dir: &str, store: &dyn SliceDB) {
    let params = ZgEncoderParams::from_dir_mont(param_dir, false, None);

    let mut rng = thread_rng();
    for _ in 0..5 {
        let mut data = vec![0u8; 1024];
        rng.fill(data.as_mut_slice());

        let raw_data: RawData = data[..].try_into().unwrap();
        let raw_blob: RawBlob = raw_data.into();

        let encoded_blob = EncodedBlob::build(&raw_blob, &params);

        let slices = (1500..2500)
            .map(|row_index| encoded_blob.get_row(row_index))
            .collect();

        store
            .put_slice(6, 0, encoded_blob.get_file_root(), slices)
            .await
            .unwrap();
    }
}
