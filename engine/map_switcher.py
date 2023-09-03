import arcade
import constants


# Stores state info for the fade-in/out effect for switching between maps.
class MapSwitch:
    def __init__(self, switch_fun, done_fun):
        self.switch_fun = switch_fun
        self.done_fun = done_fun
        self.fade_in = True
        self.alpha = 0

    def tick(self):
        step = 255 * constants.TICK_S / constants.FADE_DURATION_SECONDS
        if self.fade_in:
            self.alpha = min(255, self.alpha + step)
            if self.alpha == 255:
                self.fade_in = False
                self.switch_fun()
        else:
            self.alpha = max(0, self.alpha - step)
            if self.alpha == 0:
                self.done_fun()

    def draw(self):
        arcade.draw_lrtb_rectangle_filled(
            0, constants.SCREEN_WIDTH, constants.SCREEN_HEIGHT, 0, (0, 0, 0, self.alpha)
        )
