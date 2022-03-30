#  Copyright 2022 Katja Schmahl
#
#  Licensed under the Apache License, Version 2.0 (the "License");
#  you may not use this file except in compliance with the License.
#  You may obtain a copy of the License at
#
#      http://www.apache.org/licenses/LICENSE-2.0
#
#  Unless required by applicable law or agreed to in writing, software
#  distributed under the License is distributed on an "AS IS" BASIS,
#  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#  See the License for the specific language governing permissions and
#  limitations under the License.

import json
import os
import pandas as pd
import plotly.io
from reports import Report
import plotly.express as px



AMOUNT_OF_TESTS_IN_TOP = 3
RESULT_PATH = "../results"


def is_coppers_file(filename):
    return ".json" in filename and "coppers_results" in filename


def get_data():
    last_execution_filename = ""
    last_execution_timestamp = 0
    for filename in os.listdir(f"{RESULT_PATH}"):
        if is_coppers_file(filename):
            with open(f"{RESULT_PATH}/{filename}", "r") as f:
                result = json.load(f)
                if result["execution_timestamp"] > last_execution_timestamp:
                    last_execution_filename = filename
                    last_execution_timestamp = result["execution_timestamp"]

    with open(f"{RESULT_PATH}/{last_execution_filename}", "r") as f:
        results = json.load(f)
    return results


def visualize_all_tests(data, n):
    bars = [test["uj"]/n for test in reversed(data)]
    x = [test["name"] for test in reversed(data)]
    fig = px.bar(x=bars, y=x, labels={"x": "Energy consumption (\u03bcJ)", "y": "Test"})
    return plotly.io.to_html(fig)


def to_commit_hash(head):
    return ''.join('{:02x}'.format(x) for x in head)


def visualize_over_time():
    all_runs = pd.DataFrame()
    for filename in os.listdir(f"{RESULT_PATH}"):
        if is_coppers_file(filename):
            with open(f"{RESULT_PATH}/{filename}", "r") as f:
                result = json.load(f)
                result["head"] = to_commit_hash(result["head"])
                new_res = pd.json_normalize(result, record_path="tests", meta=["execution_timestamp", "commit_timestamp", "head"])
                n = float(result["number_of_repeats"])
                new_res['uj'] = new_res['uj']/n
                new_res['us'] = new_res['uj']/n
                all_runs = pd.concat([all_runs, new_res], axis=0)
    all_runs = all_runs.sort_values(by="execution_timestamp")
    all_runs = all_runs.sort_values(by="commit_timestamp")

    i = -1
    last_timestamp = 0
    sequential_index = []
    tick_vals = []
    tick_texts = []
    for test in all_runs.iterrows():
        if last_timestamp < test[1]["execution_timestamp"]:
            i += 1
            last_timestamp = test[1]["execution_timestamp"]
            tick_vals.append(i)
            text = ""
            for elem in test[1]["head"]:
                text = text + elem
            tick_texts.append(text)
        sequential_index.append(i)
    all_runs = all_runs.assign(sequential_index=sequential_index)
    # all_runs["timestamp"] = pd.to_datetime(all_runs['timestamp'], unit='s')

    fig = px.line(all_runs, x="sequential_index", y="uj", color="name", markers="name", labels={"sequential_index": "Commit", "\u03bcj": "Energy consumption (\u03bcJ)"})

    fig.update_layout(
        xaxis=dict(
            tickmode='array',
            tickvals=tick_vals,
            ticktext=tick_texts
        )
    )

    return plotly.io.to_html(fig)


def comparison_to_last(data):
    last_execution_filename = ""
    last_execution_timestamp = 0
    n = float(data["number_of_repeats"])
    for filename in os.listdir(f"{RESULT_PATH}"):
        if is_coppers_file(filename):
            with open(f"{RESULT_PATH}/{filename}", "r") as f:
                result = json.load(f)
                if result["execution_timestamp"] > last_execution_timestamp and last_execution_timestamp < data["execution_timestamp"]:
                    last_execution_filename = filename
                    last_execution_timestamp = result["execution_timestamp"]

    with open(f"{RESULT_PATH}/{last_execution_filename}", "r") as f:
        last_result = json.load(f)
    change_overall = data["total_consumption"]/n - last_result["total_consumption"]/n

    comparison_data = []
    for test in data["tests"]:
        test_before = [t for t in last_result["tests"] if t["name"] == test["name"]][0]
        n_before = last_result["number_of_repeats"]
        test["uj"] = test["uj"]/n
        test["us"] = test["us"]/n
        test_before["uj"] = test_before["uj"]/n_before
        test_before["us"] = test_before["us"]/n_before

        comparison_data.append([
            test["name"],
            test_before["uj"],
            test["uj"],
            test["uj"] - test_before["uj"],
            (test["uj"] - test_before["uj"]) / test_before["uj"] * 100,
            test_before["us"],
            test["us"],
            test["us"] - test_before["us"],
            (test["us"] - test_before["us"]) / test_before["us"] * 100,

        ])
    df = pd.DataFrame(comparison_data, columns=["Name", f"Usage (\u03bcJ) before", f"Usage (\u03bcJ) new",
                                                f"Change usage (\u03bcJ)", "Change usage (%)", f"Time (\u03bcs) new",
                                                f"Time (\u03bcs) before", f"Change time (\u03bcs)", "Change time (%)"])
    return change_overall, df.to_html()


def main():
    rep = Report("template")
    results = get_data()
    amount_of_results = len([filename for filename in os.listdir(RESULT_PATH) if "json" in filename])

    if amount_of_results > 2:
        rep.jinja['over_time'] = True
        rep.jinja['plot_energy_over_time'] = visualize_over_time()

    sorted_tests = sorted(results["tests"], reverse=True, key=lambda item: item["uj"])
    n = float(results["number_of_repeats"])
    rep.jinja['amount_top'] = AMOUNT_OF_TESTS_IN_TOP
    rep.jinja['most_energy_consuming_names'] = [sorted_tests[i]['name'] for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    rep.jinja['most_energy_consuming_usages'] = [sorted_tests[i]['uj']/n for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    rep.jinja['least_energy_consuming_names'] = [sorted_tests[-(i+1)]['name'] for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    rep.jinja['least_energy_consuming_usages'] = [sorted_tests[-(i+1)]['uj']/n for i in range(AMOUNT_OF_TESTS_IN_TOP)]

    if amount_of_results > 1:
        rep.jinja["compare_to_last"] = True
        rep.jinja["overall_change"], rep.jinja["comparison_table"] = comparison_to_last(results)

    rep.jinja['all_tests_plot'] = visualize_all_tests(sorted_tests, n)

    rep.create_report(onweb=False)
    print("------- Generated report of energy consumption results -------")


if __name__ == "__main__":
    main()
