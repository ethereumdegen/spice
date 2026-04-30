---
name: getNutrition
description: Get nutritional information for a food item
parameters:
  - name: food
    type: string
    description: The food item to look up nutrition info for
    required: true
---
# getNutrition

Looks up detailed nutritional information for the specified food item,
including calories, protein, fat, carbohydrates, and key micronutrients.
