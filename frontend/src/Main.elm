module Main exposing (Model, main)

import Browser
import Browser.Dom as Dom
import Browser.Events as Events
import Debug
import Dict exposing (Dict)
import Element
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Html
import Html.Attributes
import Http
import Json.Decode exposing (Decoder, dict, field, list, map2, map4, map6, maybe, string)
import Json.Encode as E
import Task



-- INBOUND PORTS
{-
   TODO:
   - Video finished
-}
-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        []



-- MODEL


type alias Model =
    { videoUrl : Maybe String
    , posterUrl : Maybe String
    , titles : List TitleShort
    , focusedTitle : FocusedTitle
    }


type FocusedTitle
    = NoFocusedTitle
    | FocusedTitleFetchError String
    | FocusedTitleFetchWait String
    | FocusedTitle TitleLong



-- MSG


type Msg
    = VideoUrlChanged String (Maybe String)
    | FetchedTitles (Result Http.Error (List TitleShort))
    | FetchedTitleDetail (Result Http.Error TitleLong)
    | PosterPressed String



-- INIT


init : () -> ( Model, Cmd Msg )
init flags =
    ( Model Nothing Nothing [] NoFocusedTitle
    , Cmd.batch
        [ getTitles
        ]
    )



-- COMMANDS


getTitles =
    Http.get
        { url = "/titles"
        , expect = Http.expectJson FetchedTitles (list decodeTitleShort)
        }



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        VideoUrlChanged videoUrl posterUrl ->
            ( { model | videoUrl = Just videoUrl, posterUrl = posterUrl }, Cmd.none )

        FetchedTitles fetchResult ->
            case fetchResult of
                Ok titles ->
                    ( { model | titles = titles }, Cmd.none )

                Err _ ->
                    Debug.todo "Implement failing title fetch"

        FetchedTitleDetail fetchResult ->
            case fetchResult of
                Ok titleDetail ->
                    ( { model | focusedTitle = FocusedTitle titleDetail }, Cmd.none )

                Err _ ->
                    Debug.todo "Implement failing title detail fetch"

        PosterPressed title ->
            ( model
            , Http.get
                { url = "/titles/" ++ title
                , expect = Http.expectJson FetchedTitleDetail decodeTitleLong
                }
            )



-- VIEW


view : Model -> Html.Html Msg
view model =
    Element.layout []
        (Element.column [ Element.width Element.fill ]
            ([ Element.el [ Element.width Element.fill ]
                (Element.html
                    (Html.node "elm-player"
                        ([]
                            ++ (case model.videoUrl of
                                    Just url ->
                                        [ Html.Attributes.attribute "src" url ]

                                    Nothing ->
                                        []
                               )
                            ++ (case model.posterUrl of
                                    Just url ->
                                        [ Html.Attributes.attribute "poster" url ]

                                    Nothing ->
                                        []
                               )
                        )
                        []
                    )
                )
             ]
                ++ (case model.focusedTitle of
                        FocusedTitle details ->
                            if List.isEmpty details.content then
                                [ Element.el
                                    [ Element.width Element.fill
                                    , Font.center
                                    , Element.padding 10
                                    ]
                                    (Element.text "No content for title")
                                ]

                            else
                                List.map
                                    (\videoContent ->
                                        Input.button
                                            [ Element.width Element.fill
                                            ]
                                            { label =
                                                Element.row
                                                    [ Element.width Element.fill ]
                                                    [ Element.image
                                                        [ Element.width (Element.px 200)
                                                        , Element.height (Element.px 128)
                                                        ]
                                                        { src = videoContent.thumbnailUrl
                                                        , description = videoContent.title
                                                        }
                                                    , Element.column
                                                        [ Element.width Element.fill
                                                        , Element.spacing 10
                                                        , Element.padding 10
                                                        ]
                                                        [ Element.paragraph
                                                            [ Font.size 24
                                                            ]
                                                            [ Element.text
                                                                (viewEpisodeNumber videoContent.metadata
                                                                    ++ " - "
                                                                    ++ videoContent.title
                                                                )
                                                            ]
                                                        , Element.paragraph
                                                            [ Font.color (Element.rgb255 180 180 180)
                                                            , Font.italic
                                                            , Font.size 16
                                                            ]
                                                            [ Element.text
                                                                (Maybe.withDefault "" videoContent.source)
                                                            ]
                                                        ]
                                                    ]
                                            , onPress = Just (VideoUrlChanged videoContent.videoUrl (Just videoContent.thumbnailUrl))
                                            }
                                    )
                                    details.content

                        _ ->
                            []
                   )
                ++ [ Element.el [ Element.width Element.fill ]
                        (Element.wrappedRow
                            [ Element.centerX
                            , Element.width Element.fill
                            ]
                            (let
                                myList =
                                    model.titles
                             in
                             List.indexedMap
                                (\n title ->
                                    Input.button
                                        [ Element.width
                                            (Element.minimum 200 (Element.fillPortion 1))
                                        ]
                                        { label =
                                            Element.image
                                                [ Element.width Element.fill
                                                ]
                                                { src = title.posterUrl
                                                , description = title.title
                                                }
                                        , onPress = Just (PosterPressed title.title)
                                        }
                                )
                                myList
                            )
                        )
                   ]
            )
        )


viewEpisodeNumber metadata =
    "S"
        ++ (Dict.get "season" metadata
                |> Maybe.withDefault "XX"
           )
        ++ "E"
        ++ (Dict.get "episode" metadata
                |> Maybe.withDefault "XX"
           )



-- REST Types


type alias TitleShort =
    { title : String
    , posterUrl : String
    }


type alias TitleLong =
    { title : String
    , posterUrl : String
    , bannerUrl : String
    , content : List VideoContent
    }


type alias VideoContent =
    { title : String
    , videoUrl : String
    , thumbnailUrl : String
    , description : String
    , metadata : Dict String String
    , source : Maybe String
    }



-- JSON Decoders


decodeTitleShort : Decoder TitleShort
decodeTitleShort =
    map2 TitleShort
        (field "title" string)
        (field "poster_url" string)


decodeTitleLong : Decoder TitleLong
decodeTitleLong =
    map4 TitleLong
        (field "title" string)
        (field "poster_url" string)
        (field "banner_url" string)
        (field "content" (list decodeVideoContent))


decodeVideoContent : Decoder VideoContent
decodeVideoContent =
    map6 VideoContent
        (field "title" string)
        (field "video_url" string)
        (field "thumbnail_url" string)
        (field "description" string)
        (field "metadata" (dict string))
        (field "source" (maybe string))



-- MAIN


main =
    Browser.element
        { init = init
        , subscriptions = subscriptions
        , view = view
        , update = update
        }
