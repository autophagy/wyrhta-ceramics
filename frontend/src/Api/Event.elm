module Api.Event exposing (Event, compareEvent, eventsDecoder, getEvents, getEventsWithLimit)

import Api exposing (ApiResource, Route(..), apiResourceDecoder)
import Api.State exposing (State, stateDecoder)
import Http
import Json.Decode exposing (Decoder, field, int, map5, maybe)
import Json.Decode.Extra exposing (datetime)
import Time exposing (Posix)
import Views.Posix exposing (comparePosix)


type alias Event =
    { id : Int
    , work : ApiResource
    , previous_state : Maybe State
    , current_state : State
    , created_at : Posix
    }


compareEvent : Event -> Event -> Order
compareEvent a b =
    comparePosix b.created_at a.created_at


getEvents : { onResponse : Result Http.Error (List Event) -> msg } -> Cmd msg
getEvents options =
    Api.get
        { route = [ Events ]
        , expect = Http.expectJson options.onResponse eventsDecoder
        }


getEventsWithLimit : Int -> { onResponse : Result Http.Error (List Event) -> msg } -> Cmd msg
getEventsWithLimit limit options =
    Api.get
        { route = [ EventsWithLimit limit ]
        , expect = Http.expectJson options.onResponse eventsDecoder
        }


eventsDecoder : Decoder (List Event)
eventsDecoder =
    Json.Decode.list eventDecoder


eventDecoder : Decoder Event
eventDecoder =
    map5 Event
        (field "id" int)
        (field "work" apiResourceDecoder)
        (field "previous_state" (maybe stateDecoder))
        (field "current_state" stateDecoder)
        (field "created_at" datetime)
