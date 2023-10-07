import arcade


def solve_music_npc(challenge: list[tuple[int, int]]):
    KEYS = {
        arcade.key.KEY_1: 0,
        arcade.key.KEY_2: 1,
        arcade.key.KEY_3: 2,
        arcade.key.KEY_4: 3,
        arcade.key.KEY_5: 4,
        arcade.key.KEY_6: 5,
        arcade.key.KEY_7: 6,
        arcade.key.KEY_8: 7,
        arcade.key.KEY_9: 8,
        arcade.key.KEY_0: 9,
        arcade.key.MINUS: 10,
        arcade.key.EQUAL: 11,
    }
    rk = {v: k for k, v in KEYS.items()}

    res = []
    for x, y in challenge:
        if x is None:
            res.extend([[]] * y)
            continue
        key = []
        if x >= 12:
            key.append(arcade.key.SPACE)
            x -= 12
        if x >= 12:
            key.append(arcade.key.ENTER)
            x -= 12

        key.append(rk[x])
        res.extend([key] * y)

    print(repr(res))


if __name__ == "__main__":
    T = 5
    solve_music_npc(
        [
            (19, T),
            (18, T),
            (19, T),
            (18, T),
            (19, T),
            (14, T),
            (17, T),
            (15, T),
            (12, 2 * T),
            (None, T),
            (3, T),
            (7, T),
            (12, T),
            (14, 2 * T),
            (None, T),
            (7, T),
            (11, T),
            (14, T),
            (15, T),
            (None, T),
            (7, T),
            (19, T),
            (18, T),
            (19, T),
            (18, T),
            (19, T),
            (14, T),
            (17, T),
            (15, T),
            (12, 2 * T),
            (None, T),
            (3, T),
            (7, T),
            (12, T),
            (14, 2 * T),
            (None, T),
            (5, T),
            (15, T),
            (14, T),
            (12, 4 * T),
        ]
    )
