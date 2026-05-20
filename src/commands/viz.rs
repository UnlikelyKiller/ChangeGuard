use crate::state::layout::Layout;
use crate::state::storage::StorageManager;
use miette::{IntoDiagnostic, Result};
use serde::Serialize;
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Serialize)]
struct VizNode {
    id: String,
    label: String,
    category: String,
    risk_score: f64,
    community: Option<i64>,
}

#[derive(Serialize)]
struct VizEdge {
    from: String,
    to: String,
    label: String,
}

pub fn execute_viz(output_path: Option<PathBuf>) -> Result<()> {
    let current_dir = env::current_dir().into_diagnostic()?;
    let layout = Layout::new(current_dir.to_string_lossy().as_ref());
    let db_path = layout.state_subdir().join("ledger.db");
    let storage = StorageManager::init(db_path.as_std_path())?;

    let cozo = storage
        .cozo
        .as_ref()
        .ok_or_else(|| miette::miette!("CozoDB not initialized. Run 'index' first."))?;

    // 1. Run Louvain community detection
    let louvain_script = "
        edges[src, dst] := *edge[src, dst, _, _, _]
        ?[node, community_id] <~ CommunityDetectionLouvain(edges[src, dst])
    ";

    let mut communities = std::collections::HashMap::new();
    if let Ok(res) = cozo.run_script(louvain_script) {
        for row in res.rows {
            if let (
                Some(cozo::DataValue::Str(node)),
                Some(cozo::DataValue::Num(cozo::Num::Int(comm))),
            ) = (row.first(), row.get(1))
            {
                communities.insert(node.to_string(), *comm);
            }
        }
    } else {
        tracing::warn!("Community detection failed. Is graph-algo enabled?");
    }

    // Fetch nodes
    let nodes_res = cozo.run_script(
        "?[id, label, category, risk_score] := *node{id, label, category, risk_score}",
    )?;
    let mut nodes = Vec::new();
    for row in nodes_res.rows {
        if let (
            Some(cozo::DataValue::Str(id)),
            Some(cozo::DataValue::Str(label)),
            Some(cozo::DataValue::Str(category)),
            Some(cozo::DataValue::Num(cozo::Num::Float(risk))),
        ) = (row.first(), row.get(1), row.get(2), row.get(3))
        {
            let community = communities.get(id.as_str()).copied();
            nodes.push(VizNode {
                id: id.to_string(),
                label: label.to_string(),
                category: category.to_string(),
                risk_score: *risk,
                community,
            });
        }
    }

    // Fetch edges
    let edges_res =
        cozo.run_script("?[source, target, relation] := *edge{source, target, relation}")?;
    let mut edges = Vec::new();
    for row in edges_res.rows {
        if let (
            Some(cozo::DataValue::Str(source)),
            Some(cozo::DataValue::Str(target)),
            Some(cozo::DataValue::Str(relation)),
        ) = (row.first(), row.get(1), row.get(2))
        {
            edges.push(VizEdge {
                from: source.to_string(),
                to: target.to_string(),
                label: relation.to_string(),
            });
        }
    }

    let html = generate_html(&nodes, &edges);

    let out = output_path.unwrap_or_else(|| layout.reports_dir().join("graph.html").into());
    if let Some(parent) = out.parent() {
        fs::create_dir_all(parent).into_diagnostic()?;
    }
    fs::write(&out, html).into_diagnostic()?;

    println!("Visualization generated at {}", out.display());
    Ok(())
}

fn generate_html(nodes: &[VizNode], edges: &[VizEdge]) -> String {
    let nodes_json = serde_json::to_string(nodes).unwrap_or_else(|_| "[]".to_string());
    let edges_json = serde_json::to_string(edges).unwrap_or_else(|_| "[]".to_string());

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>ChangeGuard Knowledge Graph</title>
    <script src="https://unpkg.com/vis-network/standalone/umd/vis-network.min.js"></script>
    <style>
        body {{
            background: #0f0f1a;
            color: #e0e0e0;
            font-family: sans-serif;
            margin: 0;
            display: flex;
            height: 100vh;
        }}
        #mynetwork {{
            flex: 1;
            height: 100%;
        }}
        #sidebar {{
            width: 300px;
            background: #1a1a2e;
            border-left: 1px solid #2a2a4e;
            padding: 15px;
            overflow-y: auto;
            display: flex;
            flex-direction: column;
        }}
        h2 {{ font-size: 1.2em; margin-top: 0; color: #4E79A7; }}
        .node-info {{ margin-bottom: 10px; padding: 10px; background: #0f0f1a; border-radius: 4px; }}
        .risk-high {{ color: #ff5555; font-weight: bold; }}
        .filter-section {{ margin-top: 20px; }}
        select, button {{ width: 100%; padding: 8px; margin-top: 5px; background: #2a2a4e; color: white; border: none; border-radius: 4px; }}
        #loading {{ position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%); font-size: 24px; color: #e0e0e0; z-index: 10; }}
    </style>
</head>
<body>
    <div id="loading">Stabilizing Graph Physics (Please Wait)...</div>
    <div id="mynetwork"></div>
    <div id="sidebar">
        <h2>Graph Info</h2>
        <div id="selection">Click a node to see details</div>
        <hr/>
        <p>Nodes: {node_count}</p>
        <p>Edges: {edge_count}</p>
        
        <div class="filter-section">
            <label for="communityFilter">Filter by Community:</label>
            <select id="communityFilter" onchange="applyFilters()">
                <option value="all">All Communities</option>
            </select>
        </div>
        
        <div class="filter-section">
            <label for="riskFilter">Filter by Risk:</label>
            <select id="riskFilter" onchange="applyFilters()">
                <option value="all">All Risks</option>
                <option value="high">High Risk (>0.7)</option>
                <option value="medium">Medium Risk (>0.3)</option>
            </select>
        </div>
        
        <div class="filter-section">
            <label for="searchInput">Search Node:</label>
            <input type="text" id="searchInput" placeholder="Type node label..." style="width: 100%; padding: 8px; margin-top: 5px; background: #2a2a4e; color: white; border: none; border-radius: 4px; box-sizing: border-box;" onkeypress="if(event.key === 'Enter') searchNode()">
            <button onclick="searchNode()">Search</button>
        </div>
        
        <button onclick="resetGraph()">Reset Graph</button>
    </div>

    <script type="text/javascript">
        const raw_nodes = {nodes_json};
        const raw_edges = {edges_json};

        const colorPalette = [
            '#4E79A7', '#F28E2B', '#E15759', '#76B7B2', '#59A14F', 
            '#EDC948', '#B07AA1', '#FF9DA7', '#9C755F', '#BAB0AC'
        ];

        // Populate community filter
        const communities = new Set();
        raw_nodes.forEach(n => {{ if (n.community !== null) communities.add(n.community); }});
        const commSelect = document.getElementById('communityFilter');
        Array.from(communities).sort((a,b) => a-b).forEach(c => {{
            const opt = document.createElement('option');
            opt.value = c;
            opt.innerHTML = `Community ${{c}}`;
            commSelect.appendChild(opt);
        }});

        const nodes = new vis.DataSet(raw_nodes.map(n => {{
            let bgColor = '#4E79A7';
            if (n.risk_score > 0.7) {{
                bgColor = '#ff5555';
            }} else if (n.community !== null && n.community !== undefined) {{
                bgColor = colorPalette[Math.abs(n.community) % colorPalette.length];
            }}
            return {{
                id: n.id,
                label: n.label,
                group: n.community,
                color: {{
                    background: bgColor,
                    border: '#2a2a4e'
                }},
                value: 1 + (n.risk_score * 5),
                title: n.label + ' (' + n.category + ')' + (n.community !== null ? ` [Comm: ${{n.community}}]` : '')
            }};
        }}));

        const edges = new vis.DataSet(raw_edges.map(e => ({{
            from: e.from,
            to: e.to,
            label: e.label,
            arrows: 'to',
            font: {{ size: 10, align: 'middle', color: '#888' }},
            color: {{ color: '#2a2a4e' }}
        }})));

        const container = document.getElementById('mynetwork');
        let data = {{ nodes: nodes, edges: edges }};
        const options = {{
            nodes: {{
                shape: 'dot',
                font: {{ color: '#e0e0e0', size: 12 }},
                borderWidth: 2
            }},
            edges: {{
                width: 1,
                smooth: {{ type: 'continuous' }}
            }},
            physics: {{
                stabilization: {{
                    enabled: true,
                    iterations: 150,
                    updateInterval: 25
                }},
                barnesHut: {{
                    gravitationalConstant: -2000,
                    centralGravity: 0.3,
                    springLength: 95
                }}
            }}
        }};
        let network = new vis.Network(container, data, options);
        
        network.on("stabilizationIterationsDone", function () {{
            network.setOptions( {{ physics: false }} );
            document.getElementById('loading').style.display = 'none';
        }});

        network.on("click", function (params) {{
            if (params.nodes.length > 0) {{
                const nodeId = params.nodes[0];
                const node = raw_nodes.find(n => n.id === nodeId);
                if (node) {{
                    document.getElementById('selection').innerHTML = `
                        <div class="node-info">
                            <b>ID:</b> ${{node.id}}<br/>
                            <b>Label:</b> ${{node.label}}<br/>
                            <b>Category:</b> ${{node.category}}<br/>
                            <b>Community:</b> ${{node.community !== null ? node.community : 'N/A'}}<br/>
                            <b>Risk Score:</b> <span class="${{node.risk_score > 0.7 ? 'risk-high' : ''}}">${{node.risk_score.toFixed(2)}}</span>
                        </div>
                    `;
                }}
            }}
        }});
        
        function applyFilters() {{
            const commVal = document.getElementById('communityFilter').value;
            const riskVal = document.getElementById('riskFilter').value;
            
            const filteredNodes = raw_nodes.filter(n => {{
                let commMatch = (commVal === 'all') || (n.community == commVal);
                let riskMatch = true;
                if (riskVal === 'high') riskMatch = n.risk_score > 0.7;
                if (riskVal === 'medium') riskMatch = n.risk_score > 0.3;
                return commMatch && riskMatch;
            }});
            
            const filteredIds = new Set(filteredNodes.map(n => n.id));
            const filteredEdges = raw_edges.filter(e => filteredIds.has(e.from) && filteredIds.has(e.to));
            
            nodes.clear();
            edges.clear();
            
            nodes.add(filteredNodes.map(n => {{
                let bgColor = '#4E79A7';
                if (n.risk_score > 0.7) {{
                    bgColor = '#ff5555';
                }} else if (n.community !== null && n.community !== undefined) {{
                    bgColor = colorPalette[Math.abs(n.community) % colorPalette.length];
                }}
                return {{
                    id: n.id,
                    label: n.label,
                    group: n.community,
                    color: {{ background: bgColor, border: '#2a2a4e' }},
                    value: 1 + (n.risk_score * 5),
                    title: n.label + ' (' + n.category + ')' + (n.community !== null ? ` [Comm: ${{n.community}}]` : '')
                }};
            }}));
            edges.add(filteredEdges);
        }}
        
        function resetGraph() {{
            document.getElementById('communityFilter').value = 'all';
            document.getElementById('riskFilter').value = 'all';
            document.getElementById('searchInput').value = '';
            applyFilters();
            network.fit();
        }}
        
        function searchNode() {{
            const query = document.getElementById('searchInput').value.toLowerCase();
            if (!query) return;
            
            const found = raw_nodes.find(n => n.label.toLowerCase().includes(query) || n.id.toLowerCase().includes(query));
            if (found) {{
                network.selectNodes([found.id]);
                network.focus(found.id, {{
                    scale: 1.5,
                    animation: {{
                        duration: 1000,
                        easingFunction: 'easeInOutQuad'
                    }}
                }});
                document.getElementById('selection').innerHTML = `
                    <div class="node-info">
                        <b>ID:</b> ${{found.id}}<br/>
                        <b>Label:</b> ${{found.label}}<br/>
                        <b>Category:</b> ${{found.category}}<br/>
                        <b>Community:</b> ${{found.community !== null ? found.community : 'N/A'}}<br/>
                        <b>Risk Score:</b> <span class="${{found.risk_score > 0.7 ? 'risk-high' : ''}}">${{found.risk_score.toFixed(2)}}</span>
                    </div>
                `;
            }} else {{
                alert("Node not found!");
            }}
        }}
    </script>
</body>
</html>"#,
        node_count = nodes.len(),
        edge_count = edges.len(),
        nodes_json = nodes_json,
        edges_json = edges_json
    )
}
