//  Copyright 2022 Katja Schmahl, Thijs Raymakers
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use pyo3::prelude::*;

const PYTHON_CODE: &str = r#"
import json
import os
from datetime import datetime

import pandas as pd
import plotly.io
from jinja2 import Environment, BaseLoader
import plotly.express as px

TEMPLATE = """
<html>
    <head>
        <link href="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-1BmE4kWBq78iYhFldvKuhfTAU6auU8tT94WrHftjDbrCEXSU1oBoqyl2QvZ6jIW3" crossorigin="anonymous">
        <link href="https://cdn.datatables.net/1.11.5/css/jquery.dataTables.min.css" rel="stylesheet">
        <style>body{ margin:0 100; background:whitesmoke; }</style>
        <meta charset="UTF-8"/>
    </head>
    <body>
        <script src="https://cdn.jsdelivr.net/npm/bootstrap@5.1.3/dist/js/bootstrap.bundle.min.js" integrity="sha384-ka7Sk0Gln4gmtz2MlQnikT1wXgYsOg+OMhuP+IlRH9sENBO0LRn5q+8nbTov4+1p" crossorigin="anonymous"></script>
        <script src="https://code.jquery.com/jquery-3.2.1.min.js"></script>
        <script src="https://cdn.datatables.net/1.11.5/js/jquery.dataTables.min.js"></script>
        <h1 class="display-3 align-center"><img src="../../logo/coppers-logo.svg" alt="Coppers logo" class="img-thumbnail" width="200"/>Coppers test energy usage</h1>

        <h2>Most energy consuming tests</h2>
        <ul class="list-group list-group-numbered">
            {%- for i in range(3) %}
              <li class="list-group-item d-flex justify-content-between align-items-start">
                <div class="ms-2 me-auto">
                  {{ most_energy_consuming_names[i] }}
                </div>
                <span class="badge bg-danger rounded-pill">{{ most_energy_consuming_usages[i] }} µJ</span>
              </li>
            {% endfor %}
        </ul>
        <h2>Least energy consuming tests</h2>
        <ul class="list-group list-group-numbered">
            {%- for i in range(amount_top) %}
              <li class="list-group-item d-flex justify-content-between align-items-start">
                <div class="ms-2 me-auto">
                  {{ least_energy_consuming_names[i] }}
                </div>
                <span class="badge bg-success rounded-pill">{{ least_energy_consuming_usages[i]}} µJ</span>
              </li>
            {% endfor %}
        </ul>

         {% if over_time %}
             <h2> Energy consumption over time </h2>
             {{ plot_energy_over_time }}
        {% endif %}

         {% if compare_to_last %}
             <h2>Comparison with previous run</h2>
             The total energy consumption changed with {{overall_change}} µJ. The change per test can be found in the table below.
             {{comparison_table}}
         {% endif %}

        <!-- *** Section 2 *** --->
        <h2>All tests energy consumptions</h2>
        {{ all_tests_plot }}
    </body>

    <script>
        $(document).ready( function () {
            $('.dataframe').DataTable();
        } );
    </script>
</html>
"""

AMOUNT_OF_TESTS_IN_TOP = 3
RESULT_PATH = "target/coppers_results"


def is_coppers_file(filename):
    return ".json" in filename and "coppers_results" in filename


def get_data():
    last_execution_filename = ""
    last_execution_timestamp = 0
    for filename in os.listdir(f"{RESULT_PATH}"):
        if is_coppers_file(filename):
            with open(f"{RESULT_PATH}/{filename}", "r") as f:
                result = json.load(f)
                if int(result["execution_timestamp"]) > int(last_execution_timestamp):
                    last_execution_filename = filename
                    last_execution_timestamp = result["execution_timestamp"]

    with open(f"{RESULT_PATH}/{last_execution_filename}", "r") as f:
        results = json.load(f)
        return results


def visualize_all_tests(data, n):
    data = sorted(data, reverse=True, key=lambda item: item["uj"])
    bars = [test["uj"] / n for test in reversed(data)]
    x = [test["name"] for test in reversed(data)]
    fig = px.bar(x=bars, y=x, labels={"x": "Energy consumption (\u03bcJ)", "y": "Test"})
    return plotly.io.to_html(fig)


def visualize_over_time():
    all_runs = pd.DataFrame()
    for filename in os.listdir(f"{RESULT_PATH}"):
        if is_coppers_file(filename):
            with open(f"{RESULT_PATH}/{filename}", "r") as f:
                result = json.load(f)
                new_res = pd.json_normalize(result, record_path="tests",
                                            meta=["execution_timestamp", "commit_timestamp", "head"])
                n = float(result["number_of_repeats"])
                new_res['uj'] = new_res['uj'] / n
                new_res['us'] = new_res['uj'] / n
                all_runs = pd.concat([all_runs, new_res], axis=0)
    all_runs = all_runs.sort_values(by="execution_timestamp", kind="mergesort")
    all_runs = all_runs.sort_values(by="commit_timestamp", kind="mergesort")

    i = -1
    last_timestamp = 0
    last_timestamp = 0
    sequential_index = []
    tick_vals = []
    tick_texts = []
    for test in all_runs.iterrows():
        if last_timestamp != test[1]["execution_timestamp"]:
            i += 1
            last_timestamp = test[1]["execution_timestamp"]
            tick_vals.append(i)
            text = test[1]["head"][:7]
            text = text + f" executed at {datetime.fromtimestamp(test[1]['execution_timestamp'])}"
            tick_texts.append(text)
        sequential_index.append(i)
    all_runs = all_runs.assign(sequential_index=sequential_index)

    fig = px.line(all_runs, x="sequential_index", y="uj", color="name", markers="name",
                  labels={"sequential_index": "Commit", "\u03bcj": "Energy consumption (\u03bcJ)"})

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
                if result["execution_timestamp"] > last_execution_timestamp & result["execution_timestamp"] < data[
                    "execution_timestamp"]:
                    last_execution_filename = filename
                    last_execution_timestamp = result["execution_timestamp"]


    with open(f"{RESULT_PATH}/{last_execution_filename}", "r") as f:
        last_result = json.load(f)
    change_overall = data["total_uj"] / n - last_result["total_uj"] / n

    comparison_data = []
    for test in data["tests"]:
        tests_before = [t for t in last_result["tests"] if t["name"] == test["name"]]
        if len(tests_before) > 0:
            test_before = tests_before[0]
            n_before = last_result["number_of_repeats"]
            test["uj"] = test["uj"] / n
            test["us"] = test["us"] / n
            test_before["uj"] = test_before["uj"] / n_before
            test_before["us"] = test_before["us"] / n_before

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
    return change_overall, df.to_html(justify='left')


def main():
    template = Environment(loader=BaseLoader).from_string(TEMPLATE)
    jinja = {}

    results = get_data()
    amount_of_results = len([filename for filename in os.listdir(RESULT_PATH) if "json" in filename])

    if amount_of_results > 2:
        jinja['over_time'] = True
        jinja['plot_energy_over_time'] = visualize_over_time()

    sorted_tests = sorted(results["tests"], reverse=True, key=lambda item: item["uj"])
    n = float(results["number_of_repeats"])
    jinja['amount_top'] = AMOUNT_OF_TESTS_IN_TOP
    jinja['most_energy_consuming_names'] = [sorted_tests[i]['name'] for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    jinja['most_energy_consuming_usages'] = [sorted_tests[i]['uj'] / n for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    jinja['least_energy_consuming_names'] = [sorted_tests[-(i + 1)]['name'] for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    jinja['least_energy_consuming_usages'] = [sorted_tests[-(i + 1)]['uj'] / n for i in range(AMOUNT_OF_TESTS_IN_TOP)]

    if amount_of_results > 1:
        jinja["compare_to_last"] = True
        jinja["overall_change"], jinja["comparison_table"] = comparison_to_last(results)

    jinja['all_tests_plot'] = visualize_all_tests(sorted_tests, n)

    report_folder = "target/coppers_report"
    generate_report(template, jinja, report_folder)
    print(f"> Generated report of energy consumption results in \"{report_folder}\"")


def generate_report(template, jinja, report_folder):
    if not os.path.isdir(report_folder):
        os.mkdir(report_folder)

    if not os.path.isdir(f"{report_folder}/images"):
        os.mkdir(f"{report_folder}/images")

    with open(f"{report_folder}/index.html", "w") as fh:
        data = template.render(jinja)
        fh.write(data)

main()
"#;

pub fn visualize() {
    Python::with_gil(|py| -> PyResult<()> {
        PyModule::from_code(py, PYTHON_CODE, "visualization.py", "visualization")?;
        Ok(())
    }).unwrap();
}
