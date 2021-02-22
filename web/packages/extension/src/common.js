function camelize(string) {
    return string.toLowerCase().replace(/[^a-zA-Z0-9]+(.)/g, (_, char) =>
        char.toUpperCase()
    );
}

export async function bindBooleanOptions(names, onChange) {
    const checkboxes = {};
    for (const name of names) {
        const checkbox = document.getElementById(name);

        const label = checkbox.nextSibling;
        label.textContent = getI18nMessage(`settings_${name}`);

        checkbox.addEventListener("click", () => {
            const value = checkbox.checked;
            options[name] = value;
            setSyncStorage({ [name]: value });
            if (onChange) {
                onChange(options);
            }
        });

        checkboxes[name] = checkbox;
    }

    const options = await getSyncStorage(names.map(camelize));
    for (const [name, value] of Object.entries(options)) {
        checkboxes[name].checked = value;
    }

    if (onChange) {
        onChange(options);
        addStorageChangeListener((changes) => {
            for (const [name, option] of Object.entries(changes)) {
                checkboxes[name].checked = option.newValue;
                options[name] = option.newValue;
            }
            onChange(options);
        });
    }
}
