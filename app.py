import streamlit as st # type: ignore # shouldn't ignore that forever
from runner import gather_raw_content

st.set_page_config(page_title="Deep Research Agent", layout="wide")
st.title("Deep Research Agent")

with st.form("search_form"):
    query = st.text_input("Enter your research topic...")
    submitted = st.form_submit_button("Run Search")

if submitted:
    with st.spinner("Searching and extracting..."):
        results = gather_raw_content(query, max_results=30)

    for page in results:
        print(page)

        # Optional UI preview
        st.subheader(page["title"])
        st.markdown(f"[{page['url']}]({page['url']})", unsafe_allow_html=True)
        st.code(page["content"][:1000] + "...")