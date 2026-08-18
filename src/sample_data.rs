use crate::model::{Article, Feed};
use chrono::{Duration, Utc};

pub fn default_feeds() -> Vec<Feed> {
    vec![
        // Fin Econ folder
        Feed::new(
            "bloomberg-markets".to_string(),
            "Bloomberg Markets".to_string(),
            "https://feeds.bloomberg.com/markets/news.rss".to_string(),
            Some("https://www.bloomberg.com/markets".to_string()),
            Some("Fin Econ".to_string()),
        ),
        Feed::new(
            "ft-alphaville".to_string(),
            "FT Alphaville".to_string(),
            "https://www.ft.com/alphaville?format=rss".to_string(),
            Some("https://www.ft.com/alphaville".to_string()),
            Some("Fin Econ".to_string()),
        ),
        Feed::new(
            "marginal-revolution".to_string(),
            "Marginal Revolution".to_string(),
            "https://marginalrevolution.com/feed".to_string(),
            Some("https://marginalrevolution.com".to_string()),
            Some("Fin Econ".to_string()),
        ),
        Feed::new(
            "econlib".to_string(),
            "Econlib".to_string(),
            "https://www.econlib.org/feed/".to_string(),
            Some("https://www.econlib.org".to_string()),
            Some("Fin Econ".to_string()),
        ),
        Feed::new(
            "calculated-risk".to_string(),
            "Calculated Risk".to_string(),
            "https://calculatedrisk.substack.com/feed".to_string(),
            Some("https://calculatedrisk.substack.com".to_string()),
            Some("Fin Econ".to_string()),
        ),
        // 2020s folder
        Feed::new(
            "ars-technica".to_string(),
            "Ars Technica".to_string(),
            "https://feeds.arstechnica.com/arstechnica/index".to_string(),
            Some("https://arstechnica.com".to_string()),
            Some("2020s".to_string()),
        ),
        Feed::new(
            "xkcd".to_string(),
            "xkcd.com".to_string(),
            "https://xkcd.com/rss.xml".to_string(),
            Some("https://xkcd.com".to_string()),
            Some("2020s".to_string()),
        ),
        Feed::new(
            "data-colada".to_string(),
            "Data Colada".to_string(),
            "https://datacolada.org/feed".to_string(),
            Some("https://datacolada.org".to_string()),
            Some("2020s".to_string()),
        ),
        // Tech & News
        Feed::new(
            "hacker-news".to_string(),
            "Hacker News".to_string(),
            "https://news.ycombinator.com/rss".to_string(),
            Some("https://news.ycombinator.com".to_string()),
            None,
        ),
        Feed::new(
            "rust-blog".to_string(),
            "Rust Blog".to_string(),
            "https://blog.rust-lang.org/feed.xml".to_string(),
            Some("https://blog.rust-lang.org".to_string()),
            None,
        ),
    ]
}

pub fn sample_articles() -> Vec<Article> {
    let now = Utc::now();
    vec![
        Article {
            id: "nse-ipo-bloomberg".to_string(),
            feed_id: "bloomberg-markets".to_string(),
            feed_title: "Bloomberg Markets".to_string(),
            title: "NSE Said to Seek Up to $55 Billion Valuation in Record India IPO".to_string(),
            author: Some("Rajesh Mascarenhas".to_string()),
            summary: Some("National Stock Exchange of India Ltd., the operator of the world's largest derivatives exchange by trading volume, is seeking a valuation of as much as 5.26 trillion rupees ($55 billion) in its planned initial public offering, according to people familiar with the matter.".to_string()),
            content: Some(r#"<p>National Stock Exchange of India Ltd., the operator of the world's largest derivatives exchange by trading volume, is seeking a valuation of as much as 5.26 trillion rupees ($55 billion) in its planned initial public offering, according to people familiar with the matter.</p>
<p>The exchange operator has proposed selling a 10% stake in the maiden share sale, the people said, asking not to be identified as the discussions are private.</p>
<h3>Key Highlights</h3>
<ul>
<li><strong>Target Valuation:</strong> Up to $55 Billion (5.26 trillion INR)</li>
<li><strong>Offering Size:</strong> 10% of total equity via secondary sale and fresh shares</li>
<li><strong>Global Position:</strong> World leader in derivatives volume for 5 consecutive years</li>
</ul>
<blockquote>"This landmark IPO marks a pivotal milestone for India's rapidly expanding financial markets and retail participation boom."</blockquote>
<p>Deliberations are ongoing, and details including the timing and valuation could change depending on regulatory clearances from the Securities and Exchange Board of India.</p>"#.to_string()),
            url: "https://www.bloomberg.com/news/articles/2026-08-18/nse-said-to-seek-up-to-55-billion-valuation-in-record-india-ipo".to_string(),
            published: Some(now - Duration::minutes(15)),
            read: false,
            starred: true,
            created_at: now - Duration::minutes(15),
        },
        Article {
            id: "hn-dont-enjoy-internet".to_string(),
            feed_id: "hacker-news".to_string(),
            feed_title: "Hacker News".to_string(),
            title: "I don't enjoy the Internet any more".to_string(),
            author: Some("btao".to_string()),
            summary: Some("Article URL: https://btao.org/posts/2026-08-17-i-dont-enjoy-the-internet/ - An essay exploring the shift from the personal web to algorithmic engagement feeds, and finding joy in RSS and small web protocols again.".to_string()),
            content: Some(r#"<p>I remember when browsing the web felt like exploring a vast, handcrafted library. Every blog had an individual personality, eccentric styling, and thoughtful writing without engagement farming.</p>
<p>Recently, opening the browser feels more like stepping into a noisy arcade where everything is optimized to trap your attention for ad impressions.</p>
<h3>Rediscovering the Calm Web</h3>
<p>Moving back to an RSS reader, plain text feeds, and direct subscriptions has completely changed my relationship with information consumption.</p>
<pre><code># The Simple Web Formula
1. Pick your sources intentionally via RSS
2. Read chronologically without algorithmic manipulation
3. Close the tab when you're done</code></pre>
<p>The internet hasn't died; it just got buried under algorithmic layers. Digging back to the fundamentals brings back the joy.</p>"#.to_string()),
            url: "https://btao.org/posts/2026-08-17-i-dont-enjoy-the-internet/".to_string(),
            published: Some(now - Duration::minutes(48)),
            read: false,
            starred: false,
            created_at: now - Duration::minutes(48),
        },
        Article {
            id: "mr-morocco-facts".to_string(),
            feed_id: "marginal-revolution".to_string(),
            feed_title: "Marginal Revolution".to_string(),
            title: "Morocco facts of the day, Morocco is not hopeless".to_string(),
            author: Some("Tyler Cowen".to_string()),
            summary: Some("For decades, South Africa was the great success story of African industrialization. Today, Morocco has become one of the premier automotive and aerospace manufacturing hubs on the Mediterranean.".to_string()),
            content: Some(r#"<p>For decades, South Africa was the great success story of African industrialization. Today, Morocco has quietly become one of the most impressive manufacturing hubs in the Mediterranean basin.</p>
<ul>
<li>Automotive exports now exceed agricultural exports by a factor of three.</li>
<li>The high-speed rail line (Al Boraq) between Tangier and Casablanca runs on renewable electricity.</li>
<li>Nearshoring from European manufacturers has accelerated significantly since 2022.</li>
</ul>
<p>Read the full analysis and policy papers linked at the Center for Global Development.</p>"#.to_string()),
            url: "https://marginalrevolution.com/marginalrevolution/2026/08/morocco-facts-of-the-day.html".to_string(),
            published: Some(now - Duration::hours(2)),
            read: false,
            starred: false,
            created_at: now - Duration::hours(2),
        },
        Article {
            id: "ft-yield-curve".to_string(),
            feed_id: "ft-alphaville".to_string(),
            feed_title: "FT Alphaville".to_string(),
            title: "China's 10-year bond yield falls to 13-month low".to_string(),
            author: Some("Global Economy Desk".to_string()),
            summary: Some("Weak economic growth and domestic deflationary pressures push institutional investors into sovereign debt as central bank signals further easing measures.".to_string()),
            content: Some(r#"<p>China's 10-year sovereign bond yield dropped to a 13-month low on Tuesday as investors sought safety amid persistent deflationary signals and soft consumer spending figures.</p>
<p>Traders noted heavy buying by domestic state commercial banks, anticipating further reserve requirement ratio (RRR) cuts later this quarter.</p>"#.to_string()),
            url: "https://www.ft.com/content/yield-curve-record-low".to_string(),
            published: Some(now - Duration::hours(3)),
            read: true,
            starred: false,
            created_at: now - Duration::hours(3),
        },
        Article {
            id: "ars-ai-boom".to_string(),
            feed_id: "ars-technica".to_string(),
            feed_title: "Ars Technica".to_string(),
            title: "AI Boom, Debt Surge Fuel Long Bond Pain | Insight with Haslinda Amin".to_string(),
            author: Some("Tech Policy Desk".to_string()),
            summary: Some("Massive capital expenditure on custom silicon, datacenter infrastructure, and grid interconnects challenges balance sheets across the semiconductor supply chain.".to_string()),
            content: Some(r#"<p>Hyperscalers are spending hundreds of billions on specialized silicon, advanced liquid cooling, and dedicated clean energy infrastructure.</p>
<p>While venture backing remains strong, debt markets are beginning to price in longer amortization cycles for generation-3 AI clusters.</p>"#.to_string()),
            url: "https://arstechnica.com/tech-policy/2026/08/ai-boom-datacenter-debt/".to_string(),
            published: Some(now - Duration::hours(4)),
            read: true,
            starred: false,
            created_at: now - Duration::hours(4),
        },
        Article {
            id: "rust-blog-update".to_string(),
            feed_id: "rust-blog".to_string(),
            feed_title: "Rust Blog".to_string(),
            title: "Announcing Rust 1.98.0: Faster Compilation and Stabilized Async Generators".to_string(),
            author: Some("The Rust Release Team".to_string()),
            summary: Some("The Rust team is happy to announce a new version of Rust, 1.98.0. Rust is a programming language empowering everyone to build reliable and efficient software.".to_string()),
            content: Some(r#"<p>The Rust team is excited to announce the release of Rust 1.98.0!</p>
<h3>What's in 1.98.0 stable</h3>
<p>Highlights of this release include:</p>
<ul>
<li><strong>Async Generators:</strong> First-class async syntax for streaming pipelines</li>
<li><strong>Parallel Frontend:</strong> 25% average compilation speedup across large codebases</li>
<li><strong>Enhanced Const Eval:</strong> Additional standard library math and formatting functions in const context</li>
</ul>
<p>To upgrade your toolchain, run:</p>
<pre><code>rustup update stable</code></pre>"#.to_string()),
            url: "https://blog.rust-lang.org/2026/08/18/Rust-1.98.0.html".to_string(),
            published: Some(now - Duration::hours(6)),
            read: false,
            starred: true,
            created_at: now - Duration::hours(6),
        },
    ]
}
