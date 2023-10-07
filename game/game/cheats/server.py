import os
import threading
from copy import deepcopy

from flask import Flask
from flask import render_template
from flask import request
from flask import send_file
from flask_autoindex import AutoIndex

from cheats.maps import request_render
from cheats.settings import settings_forms
from cheats.settings import get_settings
from cheats.settings import update_settings


def run_cheats_server(port: int) -> threading.Thread:
    app = Flask(__name__)
    app.config["WTF_CSRF_ENABLED"] = False
    app.config["TEMPLATES_AUTO_RELOAD"] = True

    static_index = AutoIndex(
        app, os.path.join(os.path.dirname(__file__), "static"), add_url_rules=False
    )

    @app.get("/static")
    @app.get("/static/<path:path>")
    def autoindex(path="."):
        return static_index.render_autoindex(path)

    @app.route("/", methods=["GET", "POST"])
    def index():
        settings = get_settings()

        forms = [
            form(request.form, **settings)
            if request.method == "POST"
            else form(**settings)
            for form in settings_forms
        ]

        if request.method == "POST":
            data = dict()
            for form in forms:
                data.update(**deepcopy(form.data))

            print(f"Updating cheat settings: {data}")

            update_settings(lambda s: s.update(**data))

        return render_template("index.html", forms=forms)

    @app.get("/map")
    def map():
        request_render()
        return send_file(
            os.path.join(os.path.dirname(__file__), "static", "current-map.png")
        )

    t = threading.Thread(target=lambda: app.run(host="localhost", port=port))
    t.start()

    return t
