table! {
    fields (id) {
        id -> Int4,
        name -> Varchar,
        synonyms -> Array<Text>,
        created_at -> Date,
        updated_at -> Date,
    }
}

table! {
    products (id) {
        id -> Int4,
        name -> Varchar,
        author -> Varchar,
        url -> Nullable<Varchar>,
        purchase_name -> Nullable<Varchar>,
        medium -> Varchar,
        description -> Text,
        source -> Text,
        reasoning -> Text,
        blurb -> Text,
        isbn -> Nullable<Varchar>,
        year_of_creation -> Varchar,
        slug -> Varchar,
        flags -> Array<Text>,
        field_id -> Nullable<Int4>,
        created_at -> Date,
        updated_at -> Date,
    }
}

table! {
    submissions (id) {
        id -> Int4,
        reference_code -> Varchar,
        name -> Varchar,
        email -> Varchar,
        bio -> Text,
        reference -> Text,
        title -> Text,
        author -> Text,
        category -> Text,
        message -> Text,
        created_at -> Date,
        updated_at -> Date,
    }
}

joinable!(products -> fields (field_id));

allow_tables_to_appear_in_same_query!(fields, products, submissions,);
