use crate::prelude::*;

struct BalancingOverview;
impl Component for BalancingOverview {
    fn render(&self) -> String {
        "balancing".into()
    }
}

pub async fn overview() -> impl IntoResponse {
    Page {
        title: "Calorie Balancing",
        children: &PageContainer {
            children: &BalancingOverview {},
        },
    }
    .render()
}
