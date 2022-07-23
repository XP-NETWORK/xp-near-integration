use test_case::test_case;
use workspaces::prelude::*;

#[test_case("bridge")]
#[test_case("xpnft")]
#[tokio::test]
async fn test_xp_bridge(contract_name: &str) -> anyhow::Result<()> {
    Ok(())
}
