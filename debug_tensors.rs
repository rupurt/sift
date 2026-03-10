use candle_core::Device;
use candle_nn::VarBuilder;

fn main() -> anyhow::Result<()> {
    let weights_path = "/home/alex/.cache/sift/models/BAAI/bge-reranker-base/main/model.safetensors";
    let device = Device::Cpu;
    let vb = unsafe { VarBuilder::from_mmaped_safetensors(&[weights_path], candle_core::DType::F32, &device)? };
    
    // VarBuilder doesn't have a way to list all keys directly easily in this version without going through the underlying storage
    // But we can try common ones
    println!("Checking roberta.embeddings.word_embeddings.weight...");
    match vb.pp("roberta").get((768, 30522), "embeddings.word_embeddings.weight") {
        Ok(_) => println!("Found with roberta. prefix"),
        Err(e) => println!("Error with roberta.: {}", e),
    }
    
    Ok(())
}
