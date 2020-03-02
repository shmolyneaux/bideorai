port module Main exposing (Model, main)

import Browser
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
import Json.Decode exposing (Decoder, dict, field, list, map2, map4, map5, string)
import Json.Encode as E



-- DEV CONSTANTS


backendPrefix =
    ""



-- OUTBOUND PORTS
{-
   TODO:
   - Change video
-}


port alert : E.Value -> Cmd msg



-- INBOUND PORTS
{-
   TODO:
   - Video finished
-}


port changeText : (String -> msg) -> Sub msg



-- SUBSCRIPTIONS


subscriptions : Model -> Sub Msg
subscriptions _ =
    Sub.batch
        [ changeText TextChanged
        ]



-- MODEL


type alias Model =
    { text : String
    , videoUrl : Maybe String
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
    = TextChanged String
    | VideoUrlChanged String (Maybe String)
    | FetchedTitles (Result Http.Error (List TitleShort))
    | FetchedTitleDetail (Result Http.Error TitleLong)
    | PosterPressed String



-- INIT


init : () -> ( Model, Cmd Msg )
init flags =
    ( Model "Init value" Nothing Nothing [] NoFocusedTitle
    , Http.get
        { url = backendPrefix ++ "/titles"
        , expect = Http.expectJson FetchedTitles (list decodeTitleShort)
        }
    )



-- UPDATE


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case msg of
        TextChanged newText ->
            ( { model | text = newText }, Cmd.none )

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
                { url = backendPrefix ++ "/titles/" ++ title
                , expect = Http.expectJson FetchedTitleDetail decodeTitleLong
                }
            )



-- VIEW


view model =
    Element.layout []
        (Element.column [ Element.width Element.fill ]
            ([ Element.el [ Element.width Element.fill ] (Element.text model.text)
             , Element.el [ Element.width Element.fill ]
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
                            List.map
                                (\videoContent ->
                                    Input.button
                                        [ Element.width Element.fill
                                        , Element.paddingEach
                                            { top = 3
                                            , right = 6
                                            , left = 6
                                            , bottom = 3
                                            }
                                        ]
                                        { label =
                                            Element.row
                                                [ Element.width Element.fill ]
                                                [ Element.image
                                                    [ Element.width (Element.px 400)
                                                    , Element.height (Element.px 255)
                                                    ]
                                                    { src = backendPrefix ++ videoContent.thumbnailUrl
                                                    , description = videoContent.title
                                                    }
                                                , Element.el [ Element.width Element.fill ]
                                                    (Element.text
                                                        (viewEpisodeNumber videoContent.metadata
                                                            ++ " - "
                                                            ++ videoContent.title
                                                        )
                                                    )
                                                ]
                                        , onPress = Just (VideoUrlChanged videoContent.videoUrl (Just videoContent.thumbnailUrl))
                                        }
                                )
                                details.content

                        _ ->
                            []
                   )
                ++ List.map
                    (\title ->
                        Input.button
                            []
                            { label =
                                Element.image []
                                    { src = backendPrefix ++ title.posterUrl
                                    , description = title.title
                                    }
                            , onPress = Just (PosterPressed title.title)
                            }
                    )
                    model.titles
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
    map5 VideoContent
        (field "title" string)
        (field "video_url" string)
        (field "thumbnail_url" string)
        (field "description" string)
        (field "metadata" (dict string))



-- MAIN


main =
    Browser.element
        { init = init
        , subscriptions = subscriptions
        , view = view
        , update = update
        }
