cd /blaze-loader

curl --header "Content-Type: application/json" --upload-file patient_minimal.json http://blaze:8080/fhir/Patient/mii-exa-person-patient-minimal
curl --header "Content-Type: application/json" --upload-file patient_full.json http://blaze:8080/fhir/Patient/mii-exa-person-patient-full
curl --header "Content-Type: application/json" --upload-file patient_pseudonymized.json http://blaze:8080/fhir/Patient/mii-exa-person-patient-pseudonymisiert

# Encounter and condition depend on each other so we need to create an empty condition first
curl --header "Content-Type: application/json" -X PUT --data '{"resourceType":"Condition","id":"mii-exa-diagnose-condition-minimal"}' http://blaze:8080/fhir/Condition/mii-exa-diagnose-condition-minimal
curl --header "Content-Type: application/json" --upload-file encounter_facility.json http://blaze:8080/fhir/Encounter/mii-exa-fall-kontakt-gesundheitseinrichtung-1
curl --header "Content-Type: application/json" --upload-file condition_example_1.json http://blaze:8080/fhir/Condition/mii-exa-diagnose-condition-minimal

# Encounter and condition depend on each other so we need to create an empty condition first
curl --header "Content-Type: application/json" -X PUT --data '{"resourceType":"Condition","id":"mii-exa-diagnose-mehrfachkodierung-primaercode"}' http://blaze:8080/fhir/Condition/mii-exa-diagnose-mehrfachkodierung-primaercode
curl --header "Content-Type: application/json" --upload-file encounter_department.json http://blaze:8080/fhir/Encounter/mii-exa-fall-kontakt-gesundheitseinrichtung-2
curl --header "Content-Type: application/json" --upload-file condition_example_2.json http://blaze:8080/fhir/Condition/mii-exa-diagnose-mehrfachkodierung-primaercode

curl --header "Content-Type: application/json" --upload-file procedure.json http://blaze:8080/fhir/Procedure/mii-exa-prozedur-procedure