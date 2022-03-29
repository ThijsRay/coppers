import json
import os
import pandas as pd
import plotly.io
from reports import Report
import plotly.express as px

AMOUNT_OF_TESTS_IN_TOP = 3
RESULT_PATH = "../results"


def get_data(filename):
    with open(f"{RESULT_PATH}/{filename}.json", "r") as f:
        results = json.load(f)
    return results


def visualize_all_tests(data, n):
    bars = [test["uj"]/n for test in reversed(data)]
    x = [test["name"] for test in reversed(data)]
    fig = px.bar(x=bars, y=x, labels={"x": "Energy consumption (uJ)", "y": "Test"})
    return plotly.io.to_html(fig)


def visualize_over_time():
    all_runs = pd.DataFrame()
    for filename in os.listdir(f"{RESULT_PATH}") if "json" in filename:
        with open(f"{RESULT_PATH}/{filename}", "r") as f:
            result = json.load(f)
            new_res = pd.json_normalize(result, record_path="tests", meta=["timestamp"])
            n = float(result["number_of_repeats"])
            new_res['uj'] = new_res['uj']/n
            new_res['us'] = new_res['uj']/n
            all_runs = pd.concat([all_runs, new_res], axis=0)
    all_runs = all_runs.sort_values(by="timestamp")
    all_runs["timestamp"] = pd.to_datetime(all_runs['timestamp'], unit='s')
    fig = px.line(all_runs, x="timestamp", y="uj", color="name", markers="name", labels={"timestamp": "Time of execution", "uj": "Energy consumption (uJ)"})
    return plotly.io.to_html(fig)


def comparison_to_last(data):
    last_execution_filename = ""
    last_execution_timestamp = 0
    n = float(data["number_of_repeats"])
    for filename in os.listdir(f"{RESULT_PATH}") if "json" in filename:
        with open(f"{RESULT_PATH}/{filename}", "r") as f:
            result = json.load(f)
            if result["timestamp"] > last_execution_timestamp and last_execution_timestamp < data["timestamp"]:
                last_execution_filename = filename
                last_execution_timestamp = result["timestamp"]

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
    df = pd.DataFrame(comparison_data, columns=["Name", f"Usage (uJ) before", f"Usage (uJ) new",
                                                f"Change usage (uJ)", "Change usage (%)", f"Time (us) new",
                                                f"Time (us) before", f"Change time (us)", "Change time (%)"])
    return change_overall, df.to_html()


def main(timestamp=1648463059):
    rep = Report("template")
    results = get_data(f"copper_results-{timestamp}")
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
