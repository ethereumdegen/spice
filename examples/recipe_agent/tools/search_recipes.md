---
name: searchRecipes
description: Search for recipes by ingredients or keywords
parameters:
  - name: ingredients
    type: string
    description: Comma-separated list of ingredients to search for
    required: true
  - name: cuisine
    type: string
    description: Optional cuisine type to filter by
    required: false
---
# searchRecipes

Searches the recipe database for recipes matching the given ingredients.
Returns a list of recipe names with brief descriptions.
