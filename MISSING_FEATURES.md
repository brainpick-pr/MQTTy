# MQTTy vs MQTT Explorer: Feature Comparison & Missing Features

This document outlines the features present in **MQTT Explorer** and compares them with the current state of **MQTTy** to identify missing features and areas for improvement.

## Feature Comparison Matrix

| Feature Category | Feature | MQTT Explorer | MQTTy | Status |
| :--- | :--- | :--- | :--- | :--- |
| **Visualization** | **Topic Hierarchy** | **Tree View**: Structured overview of topics. | **Flat List**: Linear feed of messages. | 🔴 Missing |
| | **Charts/Plots** | Plot numeric topics over time. | No plotting capabilities. | 🔴 Missing |
| | **Diff View** | Visual diff between current and previous message. | No diff view. | 🔴 Missing |
| | **Message History** | History retained per topic. | Global list per subscription tab. | 🟡 Partial |
| **Data Handling** | **JSON Support** | JSON Formatter & Editor. | Basic text preview (first 100 chars). | 🔴 Missing |
| | **Search/Filter** | Search and filter topics. | No search functionality. | 🔴 Missing |
| | **Copy/Paste** | Advanced copy support (values, topics). | Basic text selection (likely). | 🟡 Partial |
| **Management** | **Recursive Delete** | Delete topics and subtopics recursively. | No delete functionality. | 🔴 Missing |
| | **Retained Topics** | View and delete retained topics. | View only (if subscribed). | 🟡 Partial |
| | **Broker Stats** | Visualizes `$SYS` topics. | No specific support. | 🔴 Missing |
| **Connectivity** | **Profiles** | Multiple connection profiles. | Supported. | 🟢 Present |
| | **TLS/Security** | Client Certificates, SNI, CA files. | Basic TLS (Default options only). | 🟡 Partial |
| | **MQTT Version** | v3.1.1, v5.0 | v3.x, v5.0 | 🟢 Present |
| **Application** | **Background Run** | No (Standard App). | **Yes** (Runs in background, resumes). | 🌟 **Advantage** |
| | **Storage** | Local storage. | Local, VCS-friendly format. | 🟢 Present |
| | **Tech Stack** | Electron (Web technologies). | Rust + GTK4 + Libadwaita (Native). | 🟢 Present |

## Detailed Missing Features

### 1. Hierarchical Topic Tree View
**MQTT Explorer** excels at visualizing the structure of MQTT topics. It builds a dynamic tree view as messages arrive, allowing users to collapse/expand branches and see the organization of their data.
*   **MQTTy Current:** Displays messages in a flat list (chronological feed) within a subscription tab.
*   **Recommendation:** Implement a `TreeView` or `ColumnView` widget that dynamically builds nodes based on topic path separators (`/`).

### 2. Data Plotting & Charting
**MQTT Explorer** automatically detects numeric values in payloads and allows users to plot them on a graph to visualize trends over time.
*   **MQTTy Current:** Displays raw payload text.
*   **Recommendation:** Add a "Chart" view mode or widget that parses numeric payloads and uses a plotting library (e.g., `plotters` or a GTK-compatible chart widget) to render graphs.

### 3. Advanced JSON Support
**MQTT Explorer** formats JSON payloads for readability and provides a structured editor.
*   **MQTTy Current:** Shows raw text preview.
*   **Recommendation:** Detect JSON content types or try parsing payloads as JSON. If valid, pretty-print the JSON in the message detail view and potentially offer a tree-based JSON viewer.

### 4. Topic Management (Delete/Retained)
**MQTT Explorer** allows users to clean up their broker by deleting retained topics, including recursive deletion of entire topic trees.
*   **MQTTy Current:** Read-only (Subscribe/Publish).
*   **Recommendation:** Add context menu actions to "Clear Retained Message" for a topic. Recursive delete would require the Tree View implementation first.

### 5. Search & Filtering
**MQTT Explorer** allows users to search for specific topics or filter the tree view.
*   **MQTTy Current:** No search UI.
*   **Recommendation:** Add a search bar to filter the message list (or the future tree view) by topic name or payload content.

### 6. Advanced TLS Configuration
**MQTT Explorer** supports loading custom Client Certificates (CRT/Key) and CA files.
*   **MQTTy Current:** Uses `ssl_options(Default::default())`.
*   **Recommendation:** Expand the Connection Profile UI to allow file selection for Client Certificate, Client Key, and CA Certificate, and pass these to the `paho-mqtt` client options.

### 7. Diff View
**MQTT Explorer** shows a visual diff between the last received message and the previous one for the same topic, highlighting changes.
*   **MQTTy Current:** No comparison logic.
*   **Recommendation:** Store the previous message for each topic and use a diffing library to highlight changes in the UI when a new message arrives.
