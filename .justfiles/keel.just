generate:
    keel generate

heartbeat:
    keel heartbeat

health *args:
    keel health {{args}}

doctor *args:
    keel doctor {{args}}

pulse:
    keel pulse

mission next *args:
    keel mission next {{args}}

flow *args:
    keel flow {{args}}

hooks install:
    keel hooks install

next:
    keel next --agent

verify story:
    keel verify run {{story}}
