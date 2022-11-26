<h1 align="center">Yew-Template</h1>

<p align="center">
    <a href="https://crates.io/crates/yew-template"><img alt="Crates.io" src="https://img.shields.io/crates/v/yew-template"></a>
    <img alt="GitHub last commit" src="https://img.shields.io/github/last-commit/INSAgenda/yew-template?color=%23347d39" alt="last commit badge">
    <img alt="GitHub" src="https://img.shields.io/github/license/INSAgenda/yew-template">
    <img alt="GitHub top language" src="https://img.shields.io/github/languages/top/INSAgenda/yew-template">
    <img alt="GitHub closed issues" src="https://img.shields.io/github/issues-closed-raw/INSAgenda/yew-template">
    <img alt="docs.rs" src="https://img.shields.io/docsrs/yew-template">
</p>

<p align="center">A crate for using separate HTML files as <a href="https://yew.rs/">Yew</a> objects, with support for seamless localization.</p>

## Getting Started

### Hello World

```html
<div>
    <p>Hello [name]!</p>
</div>
```

```rust
let html = template_html!("templates/hello.html", name="World");
```

The code above will actually compile to the following code:

```rust
let html = yew::html! {
    <div>
        <p>{"Hello World!"}</p>
    </div>
};
```

## Usage

- [Variables](#variables)
- [Attributes](#attributes)
- [Struct fields](#struct-fields)
- [Expressions](#expressions)
- [Example: Yew callbacks](#example-with-yew-callbacks)
- [Optional variables](#optional-variables)
- [Optional elements](#optional-elements)
- [Iterators](#iterators)
- [Minimizing bloat](#minimizing-bloat)
- [Virtual elements](#virtual-elements)
- [Localization](#localization)
- [Config](#config)
- [Features](#features)
- [Security Notes](#security-notes)

### Variables

```rust
let name = "World";
let html = template_html!("templates/hello.html", name);
```

Would compile to:

```rust
let name = "World";
let html = yew::html! {
    <div>
        <p>{"Hello "}{name}{"!"}</p>
    </div>
};
```

When the name of your variable isn't the same as the name in the template, you can use the following syntax:

```rust
let other_name = "Yew";
let html = template_html!("templates/hello.html", name=other_name);
```

### Attributes

```html
<div style=[style]>
   <p>Hello [name]!</p>
</div>
```

```rust
let html = template_html!(
    "templates/hello.html",
    name="Yew",
    style="color: red;"
);
```

Yew-template supports a `format!`-like syntax in attributes, allowing you to do the following:

```html
<div style="background-color: [bg_color]; color: [text_color];">
   Yew is cool
</div>
```

### Struct fields

Sometimes you want to pass many struct fields as variables to your template, but destructuring the struct would be too verbose.
As when using the actual yew macro, you can just pass the struct and access its fields from the template:

```html
<div>
   <p>Hello [person.first_name] [person.last_name]!</p>
</div>
```

```rust
struct Person {
    first_name: String,
    last_name: String,
}

let person = Person {
    first_name: "Edouard".to_string(),
    last_name: "Foobar".to_string()
};
let html = template_html!("templates/fields.html", person);
```

### Expressions

```rust
let name_reversed = String::from("dlroW");
let html = template_html!(
    "templates/hello.html",
    name = {
        let mut name = name_reversed.into_bytes();
        name.reverse();
        let name = String::from_utf8(name).unwrap();
        name
    }
);
```

Which will also display `Hello World!` as the Yew-code output is as follows:

```rust
let name_reversed = String::from("dlroW");
let html = yew::html! {
    <div>
        <p>
            {"Hello "}{{
            let mut name = name_reversed.into_bytes();
            name.reverse();
            let name = String::from_utf8(name).unwrap();
            name
            }}{"!"}
        </p>
    </div>
};
```

Note that the curly brackets around expressions are required for expressions.

### Example with Yew callbacks

```html
<div onclick=[onclick]>
   <p>Hello [name]!</p>
</div>
```

```rust
let link = ctx.link();
let html = template_html!(
    "templates/hello.html",
    name="World",
    onclick={link.callback(|_| Msg::AddOne)}
);
```

### Optional variables

Optional variables are marked with an `opt_` prefix or an `_opt` suffix, at your option.
Their value is expected to be an `Option<T>`.

Optional variables work with optional html elements. Mark an element with the `opt` attribute to make it optional. An optional element will only be rendered if *ALL* the optional variables it contains are `Some`. Note that variables contained by smaller optional elements are excluded from this requirement.

```html
<div>
    <p>Hello [name]!</p>
    <div opt>
        <h2>Age</h2>
        <p>You are [opt_age] years old!</p>
    </div>
</div>
```

In the example above, the `div` block will not be shown if `opt_age` is `None`.

Let's see how optional elements can be nested.

```html
<div>
    <p>Hello [name]!</p>
    <div opt>
        <h2>Age</h2>
        <p>You are [opt_age] years old!</p>
        <p opt>And you are born in [opt_birth_city].</p>
    </div>
</div>
```

Here, both `opt_age` and `opt_birth_city` are optional. `opt_age` would be displayed even if `opt_birth_city` is `None`. However, if `opt_age` is `None`, `opt_birth_city` will not be displayed regardless of its value.

From the Rust side, there is no usage difference. Note that curly brackets are required (for now).

```rust
let opt_age: Option<u8> = Some(20);
let opt_birth_city: Option<String> = None;
let html = template_html!(
    "templates/opt.html",
    name="John",
    opt_age,
    opt_birth_city
);
```

In the generated Yew code, `if let` expressions are used. As a result, optional variables based on expressions behave differently as they are only evaluated once for each optional element using them.

### Optional elements

Sometimes optional variables are not suitable for making an element optional. You might need a logic that is more complex than just checking if a variable is `Some` or `None`. In this case, you can use optional elements.

Elements can be given a `present-if` attribute. The value will be evaluated at runtime as a boolean expression. If the expression is `true`, the element will be rendered. Otherwise, it will be skipped.

```html
<div present-if=[condition]>
    <p>1+1 = 3</p>
</div>
<div present-if=![condition]> <!-- Negation is supported -->
    <p>1+1 != 3</p>
</div>
```

```rust
let html = template_html!("templates/present_if.html", condition={ 1+1==3 });
```

### Iterators

Iterators work similarly to optional variables. The iterator variables are marked with an `iter_` prefix or an `_iter` suffix, at your option.
The looping html element is marked with the `iter` attribute. The element will reproduce until one of the iterators it depends on is empty.

```html
<div>
    <h2>Contributors:</h2>
    <ul>
        <li iter>[contributors_iter] ([commits_iter] commits)</li>
    </ul>
</div>
```

```rust
let contributors = vec!["John", "Jane", "Jack"]; // Owned values need to be declared as `let` or they would be freed before the template is rendered.
let html = template_html!(
    "templates/iter.html",
    contributors_iter = {contributors.iter()},
    commits_iter = {[42, 21, 7].iter()}
);
```

The code above will act as the following for Yew:

```rust
let contributors = vec!["John", "Jane", "Jack"];
let html = yew::html! {
    <div>
        <h2>{"Contributors:"}</h2>
        <ul>
            {{
                let mut contributors_iter = { contributors.iter() };
                let mut commits_iter = { [42, 21, 7].iter() };
                let mut fragments = Vec::new();
                while let (Some(contributor), Some(commits)) = (contributors_iter.next(), commits_iter.next()) {
                    fragments.push(html! { <li>{contributor}{" ("}{commits}{" commits)"}</li> });
                }
                fragments.into_iter().collect::<Html>()
            }}
        </ul>
   </div>
};
```

As of now, Yew item references in lists are not supported. This will be inmplemented in the future as the Yew documentation recommends, though the performance impact has been found to be negligible in most cases.

### Minimizing bloat

The whole point of using this crate is making your code more readable than when using Yew directly. However, you will still find yourself writing lines of code that do not carry that much meaning. We already saw that `variable_ident=variable_ident` can be shortened to `variable_ident`. But it could even be completely omitted! Add `...` at the end of your macro call to tell that undefined variables should be retrieved from local variables with the same name. Taking the "Hello world" example:

```html
<div>
    <p>Hello [name]!</p>
</div>
```

```rust
let name = "World";
let html = template_html!("templates/hello.html", ...);
```

This behavior is disabled by default because missing variables are often mistakes. If you want to enable it without have to add `...` to every macro call, please set `auto_default` to true in your [config](#config).

### Virtual elements

Yew-template often requires you to add attributes on html elements such as `iter`, `opt` or `present-if`. In rare cases, you don't have any suitable element to add these attributes to, and adding a wrapper element would break your CSS. In this case, you can use virtual elements. The virtual elements tag will be removed from the final HTML but it allows you to add special attributes where they are needed.

```html
<virtual opt>
    [opt_name]
</virtual>
```

```rust
let opt_name = Some("John".to_string());
let html = template_html!("templates/virtual.html", opt_name);
```

On Yew side, this will be seen as:

```rust
let opt_name = Some("John".to_string());
let html = yew::html! {
   <>
      if let Some(opt_name) = opt_name { {opt_name} }
  </>
};
```

And Yew will produce the following HTML:

```html
John
```

### Localization

Yew-template supports localization. It is able to extract localization data from `.po` files and automatically embed them in the generated code. Enabling this feature is as simple as putting `.po` files in a directory.

The `i18n` cargo feature needs to be enabled (it is enabled by default).

By default, the locale directory is set to `locales`. You can change this by setting `locale_directory` in your [config](#config). Yew template will automatically generate an up-to-date `.pot` file in this directory. Use it in your translation software as a template to generate `.po` files.

When done translating, put your `.po` files in the locale directory. Support for the added locales will automatically be enabled.

In order to select the locale to be rendered at runtime, you need to pass a `locale` variable to template-html macro calls. This variable will be matched against the filenames of the `.po` files in the locale directory (exluding the `.po` extension). If no match is found, the string will be left as it appears in your template.

Instead of using a `locale` variable, you can decide to evaluate any Rust expression. See the `locale_code` option in the [config](#config) section.

Yew-template prevents code injection from localized strings. This is done by escaping double quotes and backslashes. It is **SAFE** to delegate translation to unknown peers. However, these strings can include variable references, which could break compilation if referenced variables are not defined. Yew-template will take care of this issue in the future.

## Config

You can specify various settings in a `yew-template.toml` file at the crate root.
This requires the `config` cargo feature to be enabled (it is enabled by default).

This is the default configuration:

```toml
# Whether to attempt to capture local variables instead of aborting when arguments required by the template are missing.
auto_default = false

# Where to look for templates (relative to crate root)
template_directory = './'

# Where to look for locales (relative to crate root)
locale_directory = './locales/'

# Rust code to evaluate as locale. Should evaluate to a &str.
# If will be inserted in generated code like this: `match locale_code {`.
locale_code = 'locale.as_str()'
```

## Features

All features are enabled by default. There currently two features:
- [`config`](#config): Allows you to use `yew-template.toml` settings
- [`i18n`](#localization): Enables support for localization

## Security Notes

- It is safe to display all kinds of strings. They will be escaped appropriately, preventing both HTML and Rust injection.
- Localized strings are harmless in the generated code, but they could break compilation.
- Do not use untrusted template files.
- Do not use untrusted `yew-template.toml` files.

License: MIT
