FROM grafana/grafana:latest

ENV GF_ANALYTICS_REPORTING_ENABLED=false

# Копируем конфигурацию провижининга
COPY ./provisioning/dashboards/dashboards.yml /etc/grafana/provisioning/dashboards/dashboards.yml

# Копируем сами дашборды в /etc/grafana/dashboards/
COPY ./provisioning/dashboards/messenger-red.json /etc/grafana/dashboards/messenger-red.json
COPY ./provisioning/dashboards/dash-test.json /etc/grafana/dashboards/dash-test.json
