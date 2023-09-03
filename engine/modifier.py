class Modifier:
    def __init__(self, min_distance: int, variable: bool):
        self.min_distance = min_distance
        self.variable = variable

    def calculate_effect(self, effect, distance):
        if not self.variable:
            return effect
        if self.min_distance == 0:
            self.min_distance = 1
        return abs((self.min_distance - distance) / self.min_distance * effect)


class HealthDamage(Modifier):
    def __init__(self, min_distance, damage):
        super().__init__(min_distance, True)
        self.damage = damage


class HealthIncreaser(Modifier):
    def __init__(self, min_distance, benefit):
        super().__init__(min_distance, True)
        self.benefit = benefit
