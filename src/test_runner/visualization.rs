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
        <h1 class="display-3 align-center">
<svg class="img-thumbnail" width="200" clip-rule="evenodd" fill-rule="evenodd" stroke-linejoin="round" stroke-miterlimit="1.4142" version="1.1" viewBox="0 0 1200 800" xml:space="preserve" xmlns="http://www.w3.org/2000/svg" xmlns:cc="http://creativecommons.org/ns#" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:xlink="http://www.w3.org/1999/xlink">
<metadata xmlns="http://www.w3.org/2000/svg" id="metadata4652"><rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#"><cc:Work xmlns:cc="http://creativecommons.org/ns#" rdf:about=""><dc:format xmlns:dc="http://purl.org/dc/elements/1.1/">image/svg+xml</dc:format><dc:type xmlns:dc="http://purl.org/dc/elements/1.1/" rdf:resource="http://purl.org/dc/dcmitype/StillImage"/><dc:title xmlns:dc="http://purl.org/dc/elements/1.1/"/></cc:Work></rdf:RDF></metadata><defs xmlns="http://www.w3.org/2000/svg" id="defs4650"><linearGradient inkscape:collect="always" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="linearGradient5476"><stop style="stop-color:#bc664d;stop-opacity:0.68137252" offset="0" id="stop5472"/><stop style="stop-color:#bc664d;stop-opacity:1" offset="1" id="stop5474"/></linearGradient>
        <linearGradient inkscape:collect="always" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" xlink:href="\#linearGradient5476" xmlns:xlink="http://www.w3.org/1999/xlink" id="linearGradient5341" x1="-1048.0258" y1="113.1838" x2="5.8839998" y2="113.1838" gradientUnits="userSpaceOnUse"/></defs><sodipodi:namedview xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd" pagecolor="\#ffffff" bordercolor="\#666666" borderopacity="1" objecttolerance="10" gridtolerance="10" guidetolerance="10" inkscape:pageopacity="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" inkscape:pageshadow="2" inkscape:window-width="1848" inkscape:window-height="1016" id="namedview4648" showgrid="false" showguides="true" inkscape:zoom="0.73" inkscape:cx="455.4377" inkscape:cy="338.32305" inkscape:window-x="72" inkscape:window-y="27" inkscape:window-maximized="1" inkscape:current-layer="svg4646"/>
    <path xmlns="http://www.w3.org/2000/svg" style="fill:#bc664d;fill-opacity:1;fill-rule:nonzero;stroke-width:1.14796317;image-rendering:auto" d="m 452.43277,529.48488 c 0,0 -127.14053,-96.92028 -145.09243,-151.56589 0,0 137.85627,-3.58899 143.17375,-148.15793 -96.62642,41.01807 -95.75862,78.83346 -177.6972,77.32935 0,0 -20.33312,-130.63233 52.59117,-208.738512 0,0 -184.50112,34.757272 -144.44584,221.512362 0,0 78.58993,176.70015 242.9192,269.27125 z" id="path4621" inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" sodipodi:nodetypes="cccccccc" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"/><g xmlns="http://www.w3.org/2000/svg" id="g4607" transform="matrix(0.70935169,0,0,0.70935169,706.57829,710.18431)" style="fill:#72463a;fill-opacity:1">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4605" style="fill:#72463a;fill-opacity:1;fill-rule:nonzero" d="m 0,-322.648 c -114.597,0 -218.172,13.779 -296.172,36.229 v -5.071 c -78,25.095 -127.681,59.959 -127.681,98.506 0,6.077 1.345,12.062 3.703,17.931 l -7.984,14.321 c 0,0 -6.413,8.359 4.935,25.999 10.01,15.554 60.164,76.438 86.628,108.32 11.367,16.348 19.083,26.413 19.757,25.44 1.061,-1.543 -7.064,-32.229 -29.639,-67.242 -10.533,-18.805 -23.358,-43.719 -30.908,-62.12 21.081,13.342 49.189,25.445 81.189,35.861 v -0.159 c 78,22.453 181.575,36.229 296.172,36.229 131.156,0 248.828,-18.046 327.828,-46.491 V -276.153 C 248.828,-304.6 131.156,-322.648 0,-322.648"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4611" transform="matrix(0.70935169,0,0,0.70935169,1016.3748,617.61321)" style="fill:#70473b;fill-opacity:1">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4609" style="fill:#70473b;fill-opacity:1;fill-rule:nonzero" d="m 0,-50.399 -13.433,-27.828 c 0.071,-1.056 0.124,-2.114 0.124,-3.175 0,-31.548 -32.805,-60.62 -87.997,-83.901 V 2.499 c 25.751,-10.864 46.645,-22.984 61.586,-36.037 -4.398,17.683 -19.437,53.455 -31.428,78.611 -19.707,35.981 -26.845,67.303 -25.929,68.853 0.584,0.978 7.307,-9.393 17.222,-26.2 C -56.783,54.85 -13.063,-7.914 -4.325,-23.901 5.574,-42.024 0,-50.399 0,-50.399"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4615" transform="matrix(0.70935169,0,0,0.70935169,1071.7043,420.60496)" style="fill:#bc664d;fill-opacity:1">
        <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4613" style="fill:#bc664d;fill-opacity:1;fill-rule:nonzero" d="M 5.7758089e-6,227.175 -88.295994,162.132 c -0.83,-2.895 -1.66,-5.787 -2.516,-8.658 l 29.002,-42.016 c 2.961,-4.274 3.558,-9.829 1.635,-14.703 -1.925,-4.85 -6.136,-8.327 -11.117,-9.179 l -49.042996,-8.321 c -1.898,-3.879 -3.89,-7.698 -5.889,-11.484 l 20.604,-47.172 c 2.119,-4.806 1.673,-10.39 -1.139,-14.7509997 -2.797,-4.383 -7.551,-6.942 -12.617,-6.74299996 l -49.77,1.809 c -2.577,-3.35600004 -5.194,-6.68000004 -7.866,-9.94600004 l 11.438,-50.5600003 c 1.159,-5.132 -0.302,-10.508 -3.879,-14.238 -3.555,-3.716 -8.722,-5.254 -13.636,-4.05 l -48.478,11.919 c -3.14,-2.775 -6.33,-5.513 -9.559,-8.196 l 1.745,-51.907 c 0.188,-5.254 -2.262,-10.247 -6.468,-13.163 -4.204,-2.934 -9.54,-3.376 -14.138,-1.196 l -45.226,21.502 c -3.64,-2.09 -7.302,-4.16 -11.005,-6.163 l -7.991,-51.148 c -0.812,-5.181 -4.146,-9.584 -8.815,-11.594 -4.655,-2.022 -9.99,-1.367 -14.077,1.71 l -40.321,30.253 c -3.97,-1.318 -7.958,-2.583 -11.996,-3.792 l -17.38,-48.504 c -1.766,-4.945 -5.842,-8.576 -10.81,-9.597 -4.953,-1.012 -10.048,0.703 -13.483,4.539 l -33.938,37.971 c -4.104,-0.471 -8.21,-0.901 -12.327,-1.259 l -26.208,-44.154 c -2.656,-4.472 -7.344,-7.193 -12.397,-7.193 -5.041,0 -9.739,2.721 -12.372,7.193 l -26.214,44.154 c -4.119,0.358 -8.244,0.788 -12.341,1.259 l -33.943,-37.971 c -3.437,-3.836 -8.551,-5.551 -13.487,-4.539 -4.966,1.035 -9.045,4.652 -10.806,9.597 l -17.393,48.504 c -4.027,1.209 -8.017,2.482 -11.997,3.792 l -40.308,-30.253 c -4.098,-3.088 -9.432,-3.741 -14.098,-1.71 -4.65,2.01 -7.997,6.413 -8.803,11.594 l -7.997,51.148 c -3.7,2.003 -7.363,4.062 -11.014,6.163 l -45.222,-21.502 c -4.592,-2.189 -9.952,-1.738 -14.135,1.196 -4.201,2.916 -6.656,7.909 -6.466,13.163 l 1.736,51.907 c -3.219,2.683 -6.403,5.421 -9.558,8.196 l -48.47,-11.919 c -4.925,-1.195 -10.086,0.334 -13.65,4.05 -3.585,3.73 -5.039,9.106 -3.885,14.238 l 11.415,50.5600003 c -2.649,3.279 -5.27,6.59 -7.839,9.94600004 l -49.771,-1.809 c -5.023,-0.14800004 -9.817,2.35999996 -12.623,6.74299996 -2.812,4.3609997 -3.237,9.9449997 -1.146,14.7509997 l 20.619,47.172 c -2.003,3.786 -3.992,7.605 -5.906,11.484 l -49.04,8.321 c -4.982,0.841 -9.183,4.316 -11.12,9.179 -1.925,4.874 -1.3,10.429 1.639,14.703 l 29.01,42.016 c -0.224,0.741 -0.43,1.49 -0.653,2.233 l -82.02101,87.122 c 0,0 -12.56,9.851 5.73,33.002 16.14,20.421 98.99301,101.376 142.75201,143.789 18.483,21.532 31.143,34.866 32.466,33.712 2.088,-1.832 -8.871,-41.006 -45.275,-86.97 -28.06,-41.239 -64.478,-104.666 -55.939,-111.977 0,0 9.714,-12.331 29.204,-21.207 0.713,0.571 -0.737,-0.565 0,0 0,0 411.314,189.736 792.846,3.21 43.583996,-7.817 69.967996,15.529 69.967996,15.529 9.099,5.276 -14.463,70.495 -33.837996,113.666 -26.353,49.023 -30.217,87.276 -27.982,88.602 1.409,0.842 10.998,-13.747 24.674,-36.966 C -75.468994,370.196 -11.674994,280.554 5.7758089e-6,258.781 c 13.2390002241911,-24.687 0,-31.606 0,-31.606"/></g><g xmlns="http://www.w3.org/2000/svg" id="g4619" transform="matrix(0.70935169,0,0,0.70935169,800.72201,553.76943)">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4617" style="fill-rule:nonzero" d="m 0,159.631 c 1.575,-1.342 2.4,-2.139 2.4,-2.139 L -132.25,144.985 C -22.348,0 65.618,116.967 74.988,129.879 v 29.752 z"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4627" transform="matrix(0.70935169,0,0,0.70935169,753.97006,573.3376)">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4625" style="fill-rule:nonzero" d="m 0,-87.016 c 0,0 41.104,-45.009 82.21,0 0,0 32.297,60.013 0,90.016 0,0 -52.85,42.009 -82.21,0 0,0 -35.232,-33.006 0,-90.016"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4631" transform="matrix(0.70935169,0,0,0.70935169,787.72668,523.45102)">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4629" style="fill:#ffffff;fill-rule:nonzero" d="m 0,0.008 c 0,17.523 -10.329,31.73 -23.07,31.73 -12.739,0 -23.069,-14.207 -23.069,-31.73 0,-17.529 10.33,-31.738 23.069,-31.738 12.741,0 23.07,14.209 23.07,31.738"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4635" transform="matrix(0.70935169,0,0,0.70935169,623.83381,568.89706)">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4633" style="fill-rule:nonzero" d="m 0,-93.046 c 0,0 70.508,-31.219 89.753,38.463 0,0 20.159,81.218 -57.902,85.802 0,0 -99.541,-19.172 -31.851,-124.265"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4639" transform="matrix(0.70935169,0,0,0.70935169,648.95408,524.27033)">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4637" style="fill:#ffffff;fill-rule:nonzero" d="m 0,0.002 c 0,18.072 -10.653,32.729 -23.794,32.729 -13.137,0 -23.792,-14.657 -23.792,-32.729 0,-18.078 10.655,-32.731 23.792,-32.731 C -10.653,-32.729 0,-18.076 0,0.002"/>
        </g><g xmlns="http://www.w3.org/2000/svg" id="g4643" transform="matrix(0.70935169,0,0,0.70935169,947.11371,776.32284)" style="fill:#bc664d;fill-opacity:1">
            <path inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" id="path4641" style="fill:#bc664d;fill-opacity:1;fill-rule:nonzero" d="m 0,-296.808 c 0,0 -14.723,58.643 -106.292,120.267 l -25.678,6.018 c 0,0 -83.066,-151.481 -200.749,19.221 0,0 36.677,-21.354 135,4.65 0,0 -45.23,69.226 -136.342,67.099 0,0 87.313,104.749 220.18,-46.554 0,0 140.455,-54.315 151.845,-170.701 z"/>
        </g>
<g xmlns="http://www.w3.org/2000/svg" id="g6972" transform="matrix(1.0385837,1.1947762,-1.2450291,0.99294866,407.08359,-388.14663)"><path id="path5091" d="m 272.80712,99.705986 c -0.55852,0.297204 -1.94611,6.740374 -3.09491,14.117174 l -2.10286,13.50327 -13.43667,-1.70831 -13.56169,-1.72779 -15.21204,95.21473 -15.10645,95.35924 56.26367,8.76196 56.26367,8.76198 14.89535,-95.64825 14.89535,-95.64821 -13.75337,-2.14183 -13.75337,-2.14181 2.23915,-14.3785 2.23918,-14.37849 -27.88174,-4.34204 c -15.37872,-2.39494 -28.4403,-4.044832 -28.89327,-3.603124 z m 23.63339,85.896624 c 6.86037,15.15527 12.58931,33.46395 12.72973,40.78538 0.15172,6.42675 -2.90834,12.09722 -9.08302,16.38619 l -5.16301,3.55009 -4.65282,-7.12771 c -8.20295,-12.29083 -10.83995,-27.4287 -6.99792,-39.7647 2.41862,-7.3071 1.89079,-8.02961 -1.38048,-2.64816 -14.06881,22.90932 -7.13865,52.41845 16.69514,71.24149 3.5449,2.72911 6.19515,5.44696 5.88664,5.78311 -0.32788,0.46118 -1.63152,3.07555 -2.84901,5.95946 -2.13465,4.66201 -2.42368,4.87312 -4.83514,3.08889 -4.82291,-3.56846 -14.87872,-17.94072 -19.05596,-27.29948 -2.96085,-6.48004 -5.17758,-9.51457 -6.92802,-9.78717 -4.75116,-0.7399 -13.7995,-8.42407 -16.377,-14.07603 -3.74688,-8.01113 -1.48102,-15.98234 8.35733,-29.81772 8.19863,-11.52949 24.70352,-30.34563 26.32889,-30.0925 0.62518,0.0973 3.87499,6.36626 7.32465,13.81886 z" inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" style="fill:#43b388;fill-opacity:1;stroke:none;stroke-width:0.1265374"/><path id="path5109" d="m 271.12608,103.61826 -2.52764,16.16768 -15.46473,-2.41775 -15.46475,-2.41776 -16.70447,106.84728 -16.70447,106.84727 63.26484,9.89081 63.26483,9.89081 16.70447,-106.84728 16.70447,-106.84728 -15.46474,-2.41774 -15.46473,-2.41775 2.52765,-16.16768 2.52765,-16.167692 -32.33536,-5.055302 -32.33537,-5.0553 z m 59.38756,136.02646 -13.95702,89.27371 -46.39421,-7.25326 -46.39422,-7.25326 13.95703,-89.2737 13.95702,-89.27372 46.39422,7.25325 46.39421,7.25326 z" inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" style="fill:#000000;stroke:none;stroke-width:0.14229628"/></g><path xmlns="http://www.w3.org/2000/svg" style="fill:#bc664d;stroke:none;stroke-width:1px;stroke-linecap:butt;stroke-linejoin:miter;stroke-opacity:1;fill-opacity:1" d="M 176.85263,297.94645 C 153.04896,133.69769 260.14875,110.86858 325.40806,98.351898 287.7558,147.47315 263.71746,195.84926 272.81689,307.09041 Z" id="path6976" inkscape:connector-curvature="0" xmlns:inkscape="http://www.inkscape.org/namespaces/inkscape" sodipodi:nodetypes="cccc" xmlns:sodipodi="http://sodipodi.sourceforge.net/DTD/sodipodi-0.dtd"/>
</svg>
Coppers test energy usage</h1>

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
    bars = [round(test["uj"] / n) for test in reversed(data)]
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
                  labels={"sequential_index": "Commit", "uj": "Energy consumption (\u03bcJ)"},
                  hover_data={'uj':':.0f'})

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
                if (result["execution_timestamp"] > last_execution_timestamp) and (result["execution_timestamp"] < data[
                    "execution_timestamp"]):
                    last_execution_filename = filename
                    last_execution_timestamp = result["execution_timestamp"]


    with open(f"{RESULT_PATH}/{last_execution_filename}", "r") as f:
        last_result = json.load(f)
    change_overall = round(data["total_uj"] / n - last_result["total_uj"] / n)

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
                round(test_before["uj"]),
                round(test["uj"]),
                round(test["uj"] - test_before["uj"]),
                round((test["uj"] - test_before["uj"]) / test_before["uj"] * 100, 1),
                round(test_before["us"]),
                round(test["us"]),
                round(test["us"] - test_before["us"]),
                round((test["us"] - test_before["us"]) / test_before["us"] * 100, 1),

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
    jinja['most_energy_consuming_usages'] = [round(sorted_tests[i]['uj'] / n) for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    jinja['least_energy_consuming_names'] = [sorted_tests[-(i + 1)]['name'] for i in range(AMOUNT_OF_TESTS_IN_TOP)]
    jinja['least_energy_consuming_usages'] = [round(sorted_tests[-(i + 1)]['uj'] / n) for i in range(AMOUNT_OF_TESTS_IN_TOP)]

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
    })
    .unwrap();
}
