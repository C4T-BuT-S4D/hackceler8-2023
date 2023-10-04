import threading
import time
from copy import deepcopy

import requests
from flask import Flask
from flask import render_template
from flask import request
from flask_bootstrap import Bootstrap

from cheats.settings import Settings
from cheats.settings import get_settings
from cheats.settings import update_settings


def run_cheats_server(port: int) -> threading.Thread:
    app = Flask(__name__)
    app.config["SECRET_KEY"] = "ke123"
    Bootstrap(app)

    @app.route("/init")
    def app_init():
        settings = Settings(**get_settings())
        data = deepcopy(settings.data)
        del data["submit_button"]
        del data["csrf_token"]

        print("update settings:", data)

        update_settings(lambda s: s.update(**data))

        return "ok"

    @app.route("/", methods=["GET", "POST"])
    def index():
        settings = Settings(**get_settings())

        if request.method == "POST":
            data = deepcopy(settings.data)
            del data["submit_button"]
            del data["csrf_token"]

            print("update settings:", data)

            update_settings(lambda s: s.update(**data))

        return render_template("index.html", form=settings)

    t = threading.Thread(target=lambda: app.run(host="localhost", port=port))
    t.start()

    while True:
        time.sleep(1)
        r = requests.get(f"http://localhost:{port}/init")
        if r.status_code == 200:
            break

    return t
