# Warehouse Model

How warehouses are represented in Waremax.

---

## Overview

Waremax models warehouses as graphs with physical locations, connections, and resources.

| Topic | Description |
|-------|-------------|
| [Graph-Based Maps](maps.md) | Map structure and representation |
| [Nodes & Edges](nodes-edges.md) | Location and path concepts |
| [Storage & Inventory](storage.md) | Racks, bins, and SKUs |
| [Stations](stations.md) | Service stations for robots |

---

## Graph Representation

The warehouse is a **directed graph**:

- **Nodes**: Physical locations
- **Edges**: Paths between locations
- **Weights**: Edge lengths (distance)

```
    [S1]----[A1]----[A2]----[S2]
      |       |       |       |
    [R1]----[R2]----[R3]----[R4]
      |       |       |       |
    [A3]----[A4]----[A5]----[A6]
```

---

## Key Components

### Nodes

Physical locations where robots can be:

- **Aisles**: Traversable corridors
- **Racks**: Storage locations
- **Stations**: Service points
- **Charging**: Charging locations
- **Maintenance**: Repair locations

### Edges

Connections between nodes:

- Have length (meters)
- Can be bidirectional or one-way
- Have capacity limits

### Storage

Inventory system:

- Racks with multiple levels
- Bins within levels
- SKUs placed in bins
- Quantity tracking

### Stations

Service points:

- Pick stations (order fulfillment)
- Drop stations (delivery)
- Inbound/outbound stations
- Charging and maintenance

---

## Coordinate System

Nodes have (x, y) coordinates:

- Origin at (0, 0)
- X increases rightward
- Y increases upward
- Units in meters

```
Y
^
|  [0,10]----[5,10]----[10,10]
|     |         |          |
|  [0,5]-----[5,5]-----[10,5]
|     |         |          |
|  [0,0]-----[5,0]-----[10,0]
+--------------------------------> X
```

---

## Design Principles

### Grid Layouts

Most warehouses use grid-like structures:

- Regular spacing
- Clear aisles
- Accessible racks

### Station Placement

Stations typically at:

- Edges of the warehouse
- Accessible from main aisles
- Near relevant storage zones

### Traffic Flow

Consider:

- Main travel corridors
- One-way aisles (optional)
- Congestion points

---

## Related

- [Map Configuration](../../user-guide/map-configuration.md)
- [Storage Configuration](../../user-guide/storage-configuration.md)
