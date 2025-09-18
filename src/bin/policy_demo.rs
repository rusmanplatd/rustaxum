use rustaxum::app::policies::examples::PolicyExamples;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    PolicyExamples::run_all_examples().await
}