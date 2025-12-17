
# Cost Analysis

This document analyzes the principal cost components of Hierarchies packages on the IOTA network.

## 1. Create Federation (Constant Cost)

The `create_federation()` operation creates an empty federation with a fixed cost.

| Operation | Cost (IOTA) |
|-----------|-------------|
| create_federation | 0.006928 |

## 2. Add Property

The `add_property()` operation adds properties to the federation. The cost scales with the number of properties.

| Operation | Number of Properties | Cost (IOTA) |
|-----------|---------------------|-------------|
| add_property | 200 | 0.282620 |

## 3. Add Accreditor

The `add_accreditor()` operation adds accreditors to the federation. The cost depends on the number of accreditors and the number of properties assigned per accreditor.

| Operation | Number of Accreditors | Properties per Accreditor | Cost (IOTA) |
|-----------|-----------------------|---------------------------|-------------|
| add_accreditor | 200 | 5 | 1.292748 |
| add_accreditor | 100 | 20 | 1.316880 |

## 4. Validate Property

The `validate_property()` operation validates a property. The cost depends on the total number of properties, number of accreditors, and properties per accreditor.

| Operation | Number of Properties | Number of Accreditors | Properties per Accreditor | Cost (IOTA) |
|-----------|---------------------|-----------------------|---------------------------|-------------|
| validate_property | 50 | 50 | 20 | 0.002 |
| validate_property | 100 | 100 | 20 | 0.003 |
| validate_property | 200 | 200 | 5 | 0.002 |






