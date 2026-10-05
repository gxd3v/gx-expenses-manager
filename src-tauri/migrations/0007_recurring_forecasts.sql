UPDATE settings SET data = json_set(data, '$.forecast_method', 'recurring');
