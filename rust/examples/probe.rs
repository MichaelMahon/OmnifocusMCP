use omnifocus_mcp::jxa::{JxaRunner, RealJxaRunner};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runner = RealJxaRunner::new();
    let result = runner
        .run_omnijs("return document.flattenedTasks.length;")
        .await?;
    println!("{result}");
    Ok(())
}
