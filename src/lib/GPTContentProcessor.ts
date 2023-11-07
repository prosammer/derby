export class GPTContentProcessor {
  private static extractContent(jsonString: string): string {
    try {
      // Parse the input string as JSON
      const data = JSON.parse(jsonString);

      // Check if 'choices' and 'delta' fields exist
      if (data.choices && data.choices[0] && data.choices[0].delta) {
        // Return the 'content' if it exists, otherwise an empty string
        return data.choices[0].delta.content || '';
      }

      return ''; // Return empty if content is not available or if the delta is empty
    } catch (error) {
      console.error('Failed to parse JSON or extract content', error);
      return '';
    }
  }

  public processChunk(chunk: string): string {
    // Find the first occurrence of '{' to ensure we start parsing the JSON from the correct position
    const jsonStartIndex = chunk.indexOf('{');
    if (jsonStartIndex === -1) return '';  // Return empty if no JSON object start is found

    // Extract JSON string starting from the first '{'
    const jsonChunk = chunk.substring(jsonStartIndex);

    // Use the static method to extract content from the chunk
    return GPTContentProcessor.extractContent(jsonChunk);
  }
}