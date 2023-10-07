import logging
import os
import threading
from copy import deepcopy
from collections import defaultdict

from flask import Flask
from flask import redirect
from flask import render_template
from flask import request
from flask import send_file
from flask import send_from_directory
from flask import url_for
from flask_autoindex import AutoIndex

from cheats.maps import request_render
from cheats.settings import get_settings
from cheats.settings import settings_forms
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
    def static_autoindex(path="."):
        return static_index.render_autoindex(path)

    @app.get("/recordings/<path:path>")
    def recordings_autoindex(path="."):
        return send_from_directory(
            os.path.join(os.path.dirname(__file__), "recordings"), path
        )

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

            logging.info(f"Updating cheat settings: {data}")

            update_settings(lambda s: s.update(**data))
            return redirect(url_for("index"))

        return render_template("index.html", forms=forms)

    @app.route("/recordings", methods=["GET", "POST"])
    def recordings():
        all_recordings = os.listdir(
            os.path.join(os.path.dirname(__file__), "recordings")
        )

        recordings_by_map = defaultdict(list)
        for recording in all_recordings:
            if recording.count("_") < 1:
                continue
            if not recording.endswith(".json"):
                continue

            recordings_by_map[recording.split("_", 1)[0]].append(recording)

        map_names = sorted(recordings_by_map.keys())

        chosen_map = request.args.get("map")
        if chosen_map is None and len(map_names) > 0:
            chosen_map = map_names[0]

        map_recordings = None
        if chosen_map is not None:
            map_recordings = recordings_by_map.get(chosen_map)
        if map_recordings is not None:
            map_recordings = sorted(map_recordings, reverse=True)

        chosen_recording = request.args.get("recording")
        if (
            chosen_recording is None
            and map_recordings is not None
            and len(map_recordings) > 0
        ):
            chosen_recording = map_recordings[0]

        if request.method == "POST":
            update_settings(lambda s: s.update(recording_filename=chosen_recording))

            logging.info(f"Set chosen recording to {chosen_recording}")
            return redirect(url_for("recordings", **request.args))

        screenshot_name = None
        if chosen_recording is not None:
            screenshot_name = chosen_recording.removesuffix(".json") + ".png"

        return render_template(
            "recordings.html",
            maps=map_names,
            chosen_map=chosen_map,
            map_recordings=map_recordings,
            chosen_recording=chosen_recording,
            screenshot_src=f"/recordings/{screenshot_name}",
        )

    @app.get("/map")
    def map():
        request_render()
        return send_file(
            os.path.join(os.path.dirname(__file__), "static", "current-map.png")
        )

    t = threading.Thread(target=lambda: app.run(host="localhost", port=port))
    t.start()

    return t
